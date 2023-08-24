use crate::{_LEAP_BONE, _LEAP_DIGIT, _LEAP_HAND, _LEAP_PALM, _LEAP_TRACKING_EVENT};

type LeapVector = [f32; 3];
type LeapQuaternion = [f32; 4];

pub struct Bone {
    pub prev_joint: LeapVector,
    pub next_joint: LeapVector,
    pub width: f32,
    pub rotation: LeapQuaternion,
}

impl Bone {
    pub fn from_raw(raw_bone: &_LEAP_BONE) -> Bone {
        unsafe {
            Bone {
                prev_joint: raw_bone.prev_joint.__bindgen_anon_1.v,
                next_joint: raw_bone.next_joint.__bindgen_anon_1.v,
                width: raw_bone.width,
                rotation: raw_bone.rotation.__bindgen_anon_1.v,
            }
        }
    }
}

pub struct Digit {
    pub finger_id: i32,
    // pub bones: [Bone; 4usize],
    pub metacarpal: Bone,
    pub proximal: Bone,
    pub intermediate: Bone,
    pub distal: Bone,
    pub is_extended: u32,
}

impl Digit {
    pub fn from_raw(raw_digit: &_LEAP_DIGIT) -> Digit {
        unsafe {
            let digit = raw_digit.__bindgen_anon_1.__bindgen_anon_1;
            Digit {
                finger_id: raw_digit.finger_id,
                metacarpal: Bone::from_raw(&digit.metacarpal),
                proximal: Bone::from_raw(&digit.proximal),
                intermediate: Bone::from_raw(&digit.intermediate),
                distal: Bone::from_raw(&digit.distal),
                is_extended: raw_digit.is_extended,
            }
        }
    }
}

pub struct Palm {
    pub position: LeapVector,
    pub orientation: LeapQuaternion,
}

impl Palm {
    pub fn from_raw(raw_palm: &_LEAP_PALM) -> Palm {
        unsafe {
            Palm {
                position: raw_palm.position.__bindgen_anon_1.v,
                orientation: raw_palm.orientation.__bindgen_anon_1.v,
            }
        }
    }
}

pub struct Hand {
    pub id: u32,
    pub palm: Palm,
    // the fingers
    pub thumb: Digit,
    pub index: Digit,
    pub middle: Digit,
    pub ring: Digit,
    pub pinky: Digit,
}

impl Hand {
    pub fn from_raw(raw_hand: &_LEAP_HAND) -> Hand {
        unsafe {
            let fingers = raw_hand.__bindgen_anon_1.__bindgen_anon_1;
            Hand {
                id: raw_hand.id,
                palm: Palm::from_raw(&raw_hand.palm),
                thumb: Digit::from_raw(&fingers.thumb),
                index: Digit::from_raw(&fingers.index),
                middle: Digit::from_raw(&fingers.middle),
                ring: Digit::from_raw(&fingers.ring),
                pinky: Digit::from_raw(&fingers.pinky),
            }
        }
    }
}

const Y_OFFSET: f32 = 120.0;
const HEIGHT: f32 = 300.0;
const FOV_X: f32 = 130.0;
const FOV_Y: f32 = 110.0;

pub struct InteractionBox {
    pub width: f32,
    pub height: f32,
    pub depth: f32,
}

impl InteractionBox {
    pub fn new() -> InteractionBox {
        InteractionBox {
            width: Y_OFFSET * f32::tan(FOV_X.to_radians() / 2.0) * 2.0,
            height: HEIGHT,
            depth: Y_OFFSET * f32::tan(FOV_Y.to_radians() / 2.0) * 2.0,
        }
    }
    pub fn normalize_point(&self, point: LeapVector) -> LeapVector {
        let normalized_x = point[0] / (self.width / 2.0);
        let normalized_y = (point[1] - Y_OFFSET) / (self.height / 2.0);
        let normalized_z = point[2] / (self.depth / 2.0);
        [normalized_x, normalized_y - 1.0, normalized_z]
    }
}

impl Default for InteractionBox {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TrackingEvent {
    pub event_id: i64,
    pub hands: Vec<Hand>,
    pub interaction_box: InteractionBox,
}

impl TrackingEvent {
    pub fn from_raw(raw_tracking_event: &_LEAP_TRACKING_EVENT) -> TrackingEvent {
        unsafe {
            let mut tracking_event = TrackingEvent {
                event_id: raw_tracking_event.tracking_frame_id,
                hands: vec![],
                interaction_box: InteractionBox::new(),
            };

            for i in 0..raw_tracking_event.nHands {
                let raw_hand = *raw_tracking_event.pHands.offset(i as isize);
                tracking_event.hands.push(Hand::from_raw(&raw_hand));
            }
            tracking_event
        }
    }
}
