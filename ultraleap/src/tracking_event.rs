pub struct Palm {
    pub position: [f32; 3],
    pub orientation: [f32; 4],
}

pub struct Hand {
    pub id: u32,
    pub palm: Palm,
}

pub struct TrackingEvent {
    pub event_id: i64,
    pub hands: Vec<Hand>,
}
