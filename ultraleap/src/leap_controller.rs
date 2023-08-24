use crate::{
    tracking_event::*, LeapCloseConnection, LeapCreateConnection, LeapGetDeviceInfo,
    LeapOpenConnection, LeapOpenDevice, LeapPollConnection, _eLeapEventType_eLeapEventType_Device,
    _eLeapEventType_eLeapEventType_Tracking, _eLeapRS_eLeapRS_Success, LEAP_CONNECTION,
    LEAP_CONNECTION_CONFIG, LEAP_CONNECTION_MESSAGE, LEAP_DEVICE, LEAP_DEVICE_INFO,
};
use log::{error, info, trace, warn};
use std::mem::{size_of, transmute, MaybeUninit};
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
                let leap_connection_config: MaybeUninit<LEAP_CONNECTION_CONFIG> =
                    MaybeUninit::zeroed();
                let mut leap_connection: LEAP_CONNECTION = ptr::null_mut();
                info!("creating and opening connection");
                let mut result =
                    LeapCreateConnection(leap_connection_config.as_ptr(), &mut leap_connection);
                if result != _eLeapRS_eLeapRS_Success {
                    error!("failed to create connection, error: {}", result);
                    return;
                }

                result = LeapOpenConnection(leap_connection);
                if result != _eLeapRS_eLeapRS_Success {
                    error!("failed to open connection, error: {}", result);
                    return;
                }

                info!("connection created and open");
                let mut running = true;
                let mut last_frame_id = 0;

                while running {
                    if let Ok(stopped) = stop_receiver.try_recv() {
                        running = !stopped;
                        info!("stop received");
                        continue;
                    }

                    trace!("polling");
                    let mut leap_connection_message: MaybeUninit<LEAP_CONNECTION_MESSAGE> =
                        MaybeUninit::zeroed();
                    result = LeapPollConnection(
                        leap_connection,
                        1000,
                        leap_connection_message.as_mut_ptr(),
                    );

                    if result != _eLeapRS_eLeapRS_Success {
                        error!("failed to poll connection, error: {:#x}", result);
                        continue;
                    }
                    let type_ = leap_connection_message.as_ptr().read_unaligned().type_;
                    if type_ == _eLeapEventType_eLeapEventType_Device {
                        let raw_device_event = *leap_connection_message
                            .as_ptr()
                            .read_unaligned()
                            .__bindgen_anon_1
                            .device_event;
                        let raw_device_ref = raw_device_event.device;
                        let device_id = raw_device_ref.id;
                        info!("device event with id {}", device_id);

                        // let mut leap_device: MaybeUninit<LEAP_DEVICE> = MaybeUninit::uninit();
                        let mut leap_device: LEAP_DEVICE = ptr::null_mut();
                        result = LeapOpenDevice(raw_device_ref, &mut leap_device);
                        if result != _eLeapRS_eLeapRS_Success {
                            error!("failed to open device, error: {}", result);
                            continue;
                        }

                        let mut leap_device_info: MaybeUninit<LEAP_DEVICE_INFO> =
                            MaybeUninit::zeroed();

                        // let mut serial: MaybeUninit<[i8; 256]> = MaybeUninit::zeroed();
                        const SERIAL_SIZE: usize = 1000;
                        let leap_device_info_size = size_of::<LEAP_DEVICE_INFO>();
                        let mut serial: [i8; SERIAL_SIZE] = [0; SERIAL_SIZE];
                        (*leap_device_info.as_mut_ptr()).serial_length = (SERIAL_SIZE - 1) as u32;
                        (*leap_device_info.as_mut_ptr()).serial = serial.as_mut_ptr();
                        (*leap_device_info.as_mut_ptr()).size = leap_device_info_size as u32;
                        result = LeapGetDeviceInfo(leap_device, leap_device_info.as_mut_ptr());
                        if result != _eLeapRS_eLeapRS_Success {
                            error!("failed to get device info, error: {}", result);
                            continue;
                        }

                        let h_fov = leap_device_info.as_ptr().read_unaligned().h_fov;
                        let v_fov = leap_device_info.as_ptr().read_unaligned().v_fov;
                        let range = leap_device_info.as_ptr().read_unaligned().range;
                        let serial_string = std::str::from_utf8_unchecked(transmute(&serial[..]));
                        info!(
                            "device info: serial: '{}' h_fov: {}, v_fov: {}, range: {}",
                            serial_string, h_fov, v_fov, range
                        );
                    }
                    if type_ == _eLeapEventType_eLeapEventType_Tracking {
                        let raw_tracking_event = *leap_connection_message
                            .as_ptr()
                            .read_unaligned()
                            .__bindgen_anon_1
                            .tracking_event;

                        if raw_tracking_event.tracking_frame_id != last_frame_id {
                            last_frame_id = raw_tracking_event.tracking_frame_id;

                            let tracking_event = TrackingEvent::from_raw(&raw_tracking_event);

                            tracking_event_sender.send(tracking_event).unwrap();
                        }
                    }

                    trace!("polled {}", type_);
                }

                LeapCloseConnection(leap_connection);
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
