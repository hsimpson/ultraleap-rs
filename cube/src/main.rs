use std::thread;
use std::time::Duration;
use ultraleap::LeapController;

fn main() {
    let mut controller = LeapController::new();
    controller.open_connection();

    let running = true;
    while running {
        match controller.get_tracking_event() {
            Some(event) => {
                println!("event_id: {}", event.event_id);
                let hand_count = event.hands.len();
                for i in 0..hand_count {
                    let hand = &event.hands[i];
                    println!("hand[{}] id: {}", i, hand.id);
                }
            }
            None => {}
        }

        thread::sleep(Duration::from_millis(20));
    }

    controller.close_connection();
}
