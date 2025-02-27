use avian2d::prelude::*;
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
            PhysicsPlugins::default(),
        ))
        .add_plugins(PhysicsDebugPlugin::default())
        .insert_resource(Gravity(Vec2::NEG_Y * 100.0))
        .insert_resource(ClearColor(Color::srgb(0.3, 0.3, 0.3)))
        .add_event::<CollisionEvent>()
        .add_systems(Startup, (setup, spawn_floor, spawn_player))
        .add_systems(Update, move_player)
        .add_systems(Update, move_camera)
        .add_systems(Update, play_collision_sound)
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Event, Default)]
struct CollisionEvent;

#[derive(Resource)]
struct CollisionSound(Handle<AudioSource>);

fn play_collision_sound(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    sound: Res<CollisionSound>,
) {
    // Play a sound once per frame if a collision occurred.
    if !collision_events.is_empty() {
        // This prevents events staying active on the next frame.
        collision_events.clear();
        commands.spawn((AudioPlayer(sound.0.clone()), PlaybackSettings::DESPAWN));
    }
}

fn move_camera(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    camera: Single<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
) {
    let (mut transform, mut projection) = camera.into_inner();
    projection.scale *= 1. - mouse_scroll.delta.y * 0.05;
    if !mouse_input.pressed(MouseButton::Left) {
        return;
    }
    let move_delta = Vec2::new(-mouse_motion.delta.x, mouse_motion.delta.y) * projection.scale;
    transform.translation += move_delta.extend(0.0);
}

const MAX_CONTROL: f32 = 1000.0;
const INPUT_CHANGE: f32 = 10.0;
fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Single<(&mut LinearVelocity, &mut AngularVelocity), With<Player>>,
    time: Res<Time>,
) {
    let (mut linear, mut angular) = query.into_inner();
    let delta_secs = time.delta_secs();
    let mut direction = Vec2::ZERO;
    let mut rotation = 0.0;
    {
        if keyboard_input.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
            if linear.y < MAX_CONTROL {
                direction.y += INPUT_CHANGE;
            }
        }
        if keyboard_input.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
            if linear.x < MAX_CONTROL {
                direction.x += INPUT_CHANGE;
            }
        }
        if keyboard_input.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
            if -linear.x < MAX_CONTROL {
                direction.x -= INPUT_CHANGE;
            }
        }
        if keyboard_input.pressed(KeyCode::KeyQ) {
            angular.0 += 0.1;
        }
        if keyboard_input.pressed(KeyCode::KeyE) {
            angular.0 += -0.1;
        }
    }
    let move_delta = 100.0 * direction * time.delta_secs();
    linear.x += move_delta.x;
    linear.y += move_delta.y;
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Name::new("Camera"), Camera2d));
    let ball_collision_sound = asset_server.load("sounds/hitHurt.ogg");
    commands.insert_resource(CollisionSound(ball_collision_sound));
}

fn spawn_floor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Name::new("Floor"),
        RigidBody::Static,
        Collider::rectangle(1000.0, 100.0),
        Mesh2d(meshes.add(Rectangle::new(1000.0, 100.0))),
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
        Name::new("Player"),
        Player,
        RigidBody::Dynamic,
        Collider::rectangle(100.0, 100.0),
        Mesh2d(meshes.add(Rectangle::new(100.0, 100.0))),
        MeshMaterial2d(materials.add(Color::BLACK)),
        Transform::from_xyz(-300.0, 0.0, 0.0),
    ));
}
