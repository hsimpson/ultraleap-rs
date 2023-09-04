use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::{
    core_pipeline::clear_color::ClearColorConfig, window::PrimaryWindow, window::WindowMode,
};
use bevy_prototype_lyon::prelude::*;
use ultraleap::LeapController;

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Draw".into(),
                        resolution: (1280., 720.).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    filter: "info,wgpu_core=warn,wgpu_hal=warn,ultraleap=debug,draw=debug".into(),
                    ..default()
                }),
        )
        .add_plugins(ShapePlugin)
        .add_state::<DrawState>()
        .add_systems(Startup, setup_ultraleap)
        .add_systems(Startup, spawn_camera)
        .add_systems(Startup, spawn_cursor)
        // .add_systems(Startup, test_spline)
        .add_systems(Update, cursor_movement)
        // .add_systems(OnEnter(DrawState::Drawing), spawn_spline)
        // .add_systems(Update, draw)
        .add_systems(Update, draw_splines)
        .run();
}

#[derive(Component)]
struct Cursor;

#[derive(Component)]
struct Spline {
    points: Vec<Vec2>,
}

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
enum DrawState {
    #[default]
    Hovering,
    Drawing,
}

fn setup_ultraleap(world: &mut World) {
    let leap_controller = LeapController::new();
    world.insert_non_send_resource(leap_controller);
}

fn spawn_camera(mut commands: Commands) {
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
    mut commands: Commands,
    mut cursor_query: Query<(&mut Transform, &mut Fill), With<Cursor>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut spline_query: Query<&mut Spline>,
    mut leap_controller: NonSendMut<LeapController>,
    drawing_current_state: Res<State<DrawState>>,
    mut drawing_next_state: ResMut<NextState<DrawState>>,
) {
    if let Ok((mut transform, mut fill)) = cursor_query.get_single_mut() {
        if let Some(tracking_event) = leap_controller.get_tracking_event() {
            let window = window_query.get_single().unwrap();

            // at least one hand is active
            if !tracking_event.hands.is_empty() {
                let hand = &tracking_event.hands[0];
                if hand.index.is_extended == 1 {
                    // distal bone of index finger is the finger tip and next_joint is the position of the tip (the distal bone has no bone after the next_joint)
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
                        // hovering
                        fill.color = Color::Rgba {
                            red: 1.0,
                            green: 0.0,
                            blue: 0.0,
                            alpha,
                        };
                        if drawing_current_state.get() == &DrawState::Drawing {
                            drawing_next_state.set(DrawState::Hovering);
                        }
                    } else {
                        // drawing
                        fill.color = Color::Rgba {
                            red: 0.0,
                            green: 1.0,
                            blue: 0.0,
                            alpha: 0.8,
                        };
                        let spline_point =
                            Vec2::new(transform.translation.x, transform.translation.y);
                        if drawing_current_state.get() == &DrawState::Hovering {
                            drawing_next_state.set(DrawState::Drawing);
                            // spawn new spline
                            let path_builder = PathBuilder::new();
                            let path = path_builder.build();

                            commands.spawn((
                                ShapeBundle { path, ..default() },
                                Stroke::new(Color::BLACK, 3.0),
                                Spline {
                                    points: vec![spline_point],
                                },
                            ));
                        } else if let Some(mut last_spline) = spline_query.iter_mut().last() {
                            last_spline.points.push(spline_point);
                        }
                    }
                }
            }
        }
    }
}

// fn draw(mut leap_controller: NonSendMut<LeapController>) {
//     if let Some(tracking_event) = leap_controller.get_tracking_event() {
//         // at least one hand is active
//         if !tracking_event.hands.is_empty() {
//             let hand = &tracking_event.hands[0];
//             // distal bone of index finger is the finger tip
//             if hand.index.is_extended == 1 {
//                 // trace!(
//                 //     "index finger tip position: {:?}",
//                 //     hand.index.distal.next_joint
//                 // );
//             }
//         }
//     }
// }

fn draw_splines(mut spline_query: Query<(&mut Path, &mut Spline)>) {
    for (mut path, spline) in spline_query.iter_mut() {
        let mut path_builder = PathBuilder::new();
        path_builder.move_to(spline.points[0]);
        for point in spline.points.iter().skip(1) {
            path_builder.line_to(*point);
            // todo!("make quadratic bezier");
        }
        *path = path_builder.build();
    }
}

fn test_spline(mut commands: Commands) {
    let mut path_builder = PathBuilder::new();
    path_builder.move_to(Vec2::new(0., 0.));
    // path_builder.cubic_bezier_to(
    //     Vec2::new(70., 70.),
    //     Vec2::new(175., -35.),
    //     Vec2::new(0., -140.),
    // );
    path_builder.quadratic_bezier_to(Vec2::new(70., 70.), Vec2::new(175., -35.));
    // path_builder.cubic_bezier_to(
    //     Vec2::new(-175., -35.),
    //     Vec2::new(-70., 70.),
    //     Vec2::new(0., 0.),
    // );
    // path_builder.close();
    let path = path_builder.build();

    commands.spawn((
        ShapeBundle {
            path,
            // transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..default()
        },
        Stroke::new(Color::BLACK, 10.0),
        // Fill::color(Color::RED),
    ));
}
