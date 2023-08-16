#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use crate::tracking_event::*;
use log::{info, trace, warn};
use std::mem::MaybeUninit;
use std::ptr;
use std::sync::mpsc::{self, *};
use std::thread;

pub struct LeapController {
    running: bool,
    polling_thread: Option<thread::JoinHandle<()>>,
    stop_sender: Option<Sender<bool>>,
    tracking_event_receiver: Option<Receiver<TrackingEvent>>,
}

impl Default for LeapController {
    fn default() -> Self {
        Self::new()
    }
}

impl LeapController {
    pub fn new() -> LeapController {
        let mut leap_controller = LeapController {
            running: false,
            polling_thread: None,
            stop_sender: None,
            tracking_event_receiver: None,
        };
        leap_controller.open_connection();
        leap_controller
    }

    fn open_connection(&mut self) {
        if self.running {
            warn!("already running");
            return;
        }
        self.running = true;

        let (stop_sender, stop_receiver) = mpsc::channel();
        let (tracking_event_sender, tracking_event_receiver) = mpsc::channel();

        self.stop_sender = Some(stop_sender);
        self.tracking_event_receiver = Some(tracking_event_receiver);
        self.polling_thread = Some(thread::spawn(move || {
            info!("start polling thread");
            unsafe {
                const CONFIG: MaybeUninit<LEAP_CONNECTION_CONFIG> = MaybeUninit::uninit();
                let mut connection: LEAP_CONNECTION = ptr::null_mut();
                let mut running = false;
                info!("creating and opening connection");
                if LeapCreateConnection(CONFIG.as_ptr(), &mut connection)
                    == _eLeapRS_eLeapRS_Success
                    && LeapOpenConnection(connection) == _eLeapRS_eLeapRS_Success
                {
                    info!("connection created and open");
                    running = true;
                }

                let mut last_frame_id = 0;

                while running {
                    trace!("polling");
                    if let Ok(stopped) = stop_receiver.try_recv() {
                        running = !stopped;
                        info!("stop received");
                    }

                    let mut msg: MaybeUninit<LEAP_CONNECTION_MESSAGE> = MaybeUninit::uninit();
                    if LeapPollConnection(connection, 1000, msg.as_mut_ptr())
                        != _eLeapRS_eLeapRS_Success
                    {
                        continue;
                    }
                    let type_ = msg.as_ptr().read_unaligned().type_;
                    if type_ == _eLeapEventType_eLeapEventType_Tracking {
                        let raw_tracking_event = *msg
                            .as_ptr()
                            .read_unaligned()
                            .__bindgen_anon_1
                            .tracking_event;

                        if raw_tracking_event.tracking_frame_id != last_frame_id {
                            last_frame_id = raw_tracking_event.tracking_frame_id;

                            let mut tracking_event = TrackingEvent {
                                event_id: raw_tracking_event.tracking_frame_id,
                                hands: vec![],
                            };

                            if raw_tracking_event.nHands > 0 {
                                for i in 0..raw_tracking_event.nHands {
                                    let raw_hand = *raw_tracking_event.pHands.offset(i as isize);
                                    let raw_palm = raw_hand.palm;
                                    let palm = Palm {
                                        position: raw_palm.position.__bindgen_anon_1.v,
                                        orientation: raw_palm.orientation.__bindgen_anon_1.v,
                                    };
                                    let hand = Hand {
                                        id: raw_hand.id,
                                        palm,
                                    };
                                    tracking_event.hands.push(hand);
                                }
                            }

                            tracking_event_sender.send(tracking_event).unwrap();
                        }
                    }

                    trace!("polled {}", type_);
                }

                LeapCloseConnection(connection);
            }
            info!("end polling thread")
        }));
    }

    fn close_connection(&mut self) {
        self.stop_sender.take().unwrap().send(true).unwrap();
        self.polling_thread.take().unwrap().join().unwrap();
    }

    pub fn get_tracking_event(&mut self) -> Option<TrackingEvent> {
        match self.tracking_event_receiver {
            Some(ref receiver) => match receiver.try_recv() {
                Ok(tracking_event) => Some(tracking_event),
                _ => None,
            },
            _ => None,
        }
    }
}

impl Drop for LeapController {
    fn drop(&mut self) {
        if self.running {
            info!("closing connection");
            self.close_connection();
        }
    }
}
