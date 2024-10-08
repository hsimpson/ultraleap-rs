use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::prelude::*;
use bevy::render::camera::ClearColorConfig;
use ultraleap::LeapController;

const TRANSLATION_FACTOR: f32 = 0.025;
const Y_OFFSET: f32 = 150.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WireframePlugin)
        .add_systems(Startup, setup_ultraleap)
        .add_systems(Startup, spawn_camera)
        .add_systems(Startup, spawn_cube)
        .add_systems(Update, cube_movement)
        .run();
}

#[derive(Component)]
struct Cube;

fn setup_ultraleap(world: &mut World) {
    let leap_controller = LeapController::new();
    world.insert_non_send_resource(leap_controller);
}

fn spawn_cube(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::srgb(0.0, 0.0, 1.0)),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        Wireframe,
        Cube {},
    ));
}

fn spawn_camera(mut commands: Commands) {
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        camera_3d: Camera3d { ..default() },
        camera: Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.1, 0.1, 0.1)),
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn cube_movement(
    // keyboard_input: Res<Input<KeyCode>>,
    mut cube_query: Query<&mut Transform, With<Cube>>,
    // time: Res<Time>,
    mut leap_controller: NonSendMut<LeapController>,
) {
    if let Ok(mut transform) = cube_query.get_single_mut() {
        if let Some(tracking_event) = leap_controller.get_tracking_event() {
            // at least one hand is active
            if !tracking_event.hands.is_empty() {
                let hand = &tracking_event.hands[0];
                let palm = &hand.palm;
                let mut translation = Vec3::from_array(palm.position);
                let rotation = Quat::from_array(palm.orientation);
                translation.y -= Y_OFFSET;
                translation *= TRANSLATION_FACTOR;
                transform.translation = translation;
                transform.rotation = rotation;
            }
        }
    }
}
