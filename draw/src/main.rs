use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::{core_pipeline::clear_color::ClearColorConfig, window::PrimaryWindow};
use bevy_prototype_lyon::{entity::ShapeBundle, prelude::*};
use ultraleap::LeapController;

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .add_plugins(DefaultPlugins.set(LogPlugin {
            filter: "info,wgpu_core=warn,wgpu_hal=warn,ultraleap=debug,draw=debug".into(),
            ..default()
        }))
        .add_plugins(ShapePlugin)
        .add_systems(Startup, setup_ultraleap)
        .add_systems(Startup, spawn_camera)
        .add_systems(Startup, spawn_cursor)
        .add_systems(Update, cursor_movement)
        .add_systems(Update, draw)
        .run();
}

#[derive(Component)]
struct Cursor;

fn setup_ultraleap(world: &mut World) {
    let leap_controller = LeapController::new();
    world.insert_non_send_resource(leap_controller);
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::rgb(0.96, 0.96, 0.88)),
        },
        ..default()
    });
}

fn spawn_cursor(mut commands: Commands) {
    let shape = shapes::Circle {
        radius: 12.0,
        ..shapes::Circle::default()
    };

    commands.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&shape),
            ..default()
        },
        Fill::color(Color::Rgba {
            red: 1.0,
            green: 0.0,
            blue: 0.0,
            alpha: 0.8,
        }),
        Stroke::new(
            Color::Rgba {
                red: 1.0,
                green: 0.0,
                blue: 0.0,
                alpha: 0.8,
            },
            3.0,
        ),
        Cursor {},
    ));
}

fn cursor_movement(
    mut cursor_query: Query<(&mut Transform, &mut Fill), With<Cursor>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut leap_controller: NonSendMut<LeapController>,
) {
    if let Ok((mut transform, mut fill)) = cursor_query.get_single_mut() {
        if let Some(tracking_event) = leap_controller.get_tracking_event() {
            let window = window_query.get_single().unwrap();

            // at least one hand is active
            if !tracking_event.hands.is_empty() {
                let hand = &tracking_event.hands[0];
                if hand.index.is_extended == 1 {
                    // distal bone of index finger is the finger tip and next_joint is the position of the tip (the distal bone has no bone after the next_joint)
                    // trace!(
                    //     "index finger tip position: {:?}",
                    //     hand.index.distal.next_joint
                    // );
                    let translation = tracking_event
                        .interaction_box
                        .normalize_point(hand.index.distal.next_joint);
                    trace!("index finger tip position (normalized): {:?}", translation);
                    transform.translation = Vec3::new(
                        translation[0] * (window.width() / 2.0),
                        translation[1] * (window.height() / 2.0),
                        0.0,
                    );

                    let z = translation[2];
                    let alpha = (z.clamp(0.0, 1.0) - 1.0).abs();
                    if z > 0.0 {
                        fill.color = Color::Rgba {
                            red: 1.0,
                            green: 0.0,
                            blue: 0.0,
                            alpha,
                        };
                    } else {
                        fill.color = Color::Rgba {
                            red: 0.0,
                            green: 1.0,
                            blue: 0.0,
                            alpha: 0.8,
                        };
                    }
                }
            }
        }
    }
}

fn draw(mut leap_controller: NonSendMut<LeapController>) {
    if let Some(tracking_event) = leap_controller.get_tracking_event() {
        // at least one hand is active
        if !tracking_event.hands.is_empty() {
            let hand = &tracking_event.hands[0];
            // distal bone of index finger is the finger tip
            if hand.index.is_extended == 1 {
                // trace!(
                //     "index finger tip position: {:?}",
                //     hand.index.distal.next_joint
                // );
            }
        }
    }
}
