pub struct Hand {
    pub id: u32,
}

pub struct TrackingEvent {
    pub event_id: i64,
    pub hands: Vec<Hand>,
}
