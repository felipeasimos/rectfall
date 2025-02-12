use bevy::asset::AssetMetaCheck;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics in web builds on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                }),
            // PhysicsPlugins::default(),
        ))
        .insert_resource(ClearColor(Color::srgb(0.3, 0.3, 0.3)))
        .add_systems(Startup, (setup, spawn_floor, spawn_player))
        .add_systems(Update, gravity)
        .add_systems(Update, move_player)
        .add_systems(Update, speed)
        .add_systems(Update, damp)
        .add_systems(Update, collider)
        .add_systems(Update, acceleration)
        .add_systems(Update, gravitational_pull)
        .add_systems(Update, move_camera)
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Speed(Vec2);

#[derive(Component)]
struct Acceleration(Vec2);

#[derive(Component)]
struct Mass(f32);

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Gravity;

#[derive(Component)]
struct Static;

#[derive(Component)]
struct Bouncy(f32);

const GRAVITY: f32 = -10.0;
fn gravity(mut query: Query<&mut Speed, (With<Gravity>, Without<Static>)>, time: Res<Time>) {
    let move_delta = Vec2::new(0.0, GRAVITY * time.delta_secs());
    for mut speed in &mut query {
        speed.0 += move_delta;
    }
}

const GRAVITATIONAL_CONSTANT: f32 = 69.0;
fn gravitational_pull(
    mut query: Query<(Option<&mut Acceleration>, &Mass, &Transform)>,
    time: Res<Time>,
) {
    let mut iter = query.iter_combinations_mut();
    while let Some([(acc1_opt, Mass(m1), transform1), (acc2_opt, Mass(m2), transform2)]) =
        iter.fetch_next()
    {
        let delta = transform2.translation - transform1.translation;
        let distance_sq: f32 = delta.length_squared();

        let f = GRAVITATIONAL_CONSTANT / distance_sq;
        let force_unit_mass = delta * f;
        if let Some(mut acc1) = acc1_opt {
            acc1.0 += force_unit_mass.xy() * *m2 * time.delta_secs();
        }
        if let Some(mut acc2) = acc2_opt {
            acc2.0 -= force_unit_mass.xy() * *m1 * time.delta_secs();
        }
    }
}

fn acceleration(mut query: Query<(&mut Speed, &Acceleration)>, time: Res<Time>) {
    query.iter_mut().for_each(|(mut speed, acceleration)| {
        speed.0 += acceleration.0 * time.delta_secs();
    });
}

fn speed(mut query: Query<(&mut Transform, &Speed)>, time: Res<Time>) {
    query.iter_mut().for_each(|(mut transform, speed)| {
        transform.translation += speed.0.extend(0.0) * time.delta_secs();
    });
}

const DAMP: f32 = 0.1;
fn damp(mut query: Query<&mut Speed>, time: Res<Time>) {
    for mut speed in &mut query {
        let move_delta = -speed.0 * DAMP * time.delta_secs();
        speed.0 += move_delta;
    }
}

fn collider(mut query: Query<(&mut Speed, &Transform, &Mesh2d), With<Collider>>) {
    let mut iter = query.iter_combinations_mut();
    while let Some([(mut speed_a, transform_a, mesh_a), (mut speed_b, transform_b, mesh_b)]) =
        iter.fetch_next()
    {}
}

fn move_camera(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mut camera: Single<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
) {
    camera.1.scale *= 1. - mouse_scroll.delta.y * 0.05;
    if !mouse_input.pressed(MouseButton::Left) {
        return;
    }
    let move_delta = Vec2::new(-mouse_motion.delta.x, mouse_motion.delta.y);
    camera.0.translation += move_delta.extend(0.0);
}

const MAX_CONTROL: f32 = 10.0;
fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Speed, With<Player>>,
    time: Res<Time>,
) {
    let mut direction = Vec2::ZERO;
    {
        let speed = query.single();
        if keyboard_input.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
            if speed.0.y < MAX_CONTROL {
                direction.y += 1.0;
            }
        }
        if keyboard_input.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
            if speed.0.x < MAX_CONTROL {
                direction.x += 1.0;
            }
        }
        if keyboard_input.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
            if -speed.0.x < MAX_CONTROL {
                direction.x -= 1.0;
            }
        }
    }
    let move_delta = 100.0 * direction * time.delta_secs();
    let mut speed = query.single_mut();
    speed.0 += move_delta;
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_floor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Player,
        Static,
        Collider,
        Mass(10.0),
        Mesh2d(meshes.add(Rectangle::new(200.0, 100.0))),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Transform::from_xyz(0.0, -300.0, 0.0),
    ));
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Player,
        Gravity,
        Collider,
        Bouncy(1.0),
        Acceleration(Vec2::ZERO),
        Speed(Vec2::ZERO),
        Mass(1.0),
        Mesh2d(meshes.add(Rectangle::new(100.0, 100.0))),
        MeshMaterial2d(materials.add(Color::BLACK)),
        Transform::from_xyz(-150.0, 0.0, 0.0),
    ));
}
