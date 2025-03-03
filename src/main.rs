use avian2d::prelude::*;
use bevy::asset::AssetMetaCheck;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;

const GRAVITY: f32 = 1000.0;
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
            PhysicsPlugins::default().set(PhysicsInterpolationPlugin::interpolate_all()),
        ))
        .add_plugins(PhysicsDebugPlugin::default())
        .insert_resource(Gravity(Vec2::NEG_Y * GRAVITY))
        .insert_resource(ClearColor(Color::srgb(0.3, 0.3, 0.3)))
        .add_systems(Startup, (setup, spawn_floor, spawn_player))
        .add_systems(Update, move_player)
        .add_systems(Update, move_camera)
        .add_systems(Update, handle_collision)
        .add_systems(FixedPostUpdate, player_fast_falling)
        .run();
}

#[derive(Component)]
struct Player {
    can_jump: bool,
    is_in_air: bool,
}

impl Default for Player {
    fn default() -> Player {
        Player {
            can_jump: false,
            is_in_air: false,
        }
    }
}

#[derive(Resource)]
struct CollisionSound(Handle<AudioSource>);

fn handle_collision(
    mut commands: Commands,
    mut collision_event_reader: EventReader<CollisionStarted>,
    mut query: Query<&mut Player>,
    sound: Res<CollisionSound>,
) {
    if collision_event_reader.is_empty() {
        return;
    }
    for CollisionStarted(entity1, entity2) in collision_event_reader.read() {
        if query.contains(*entity1) {
            query.single_mut().can_jump = true;
            query.single_mut().is_in_air = false;
        }
        if query.contains(*entity2) {
            query.single_mut().can_jump = true;
            query.single_mut().is_in_air = false;
        }
    }

    commands.spawn((AudioPlayer(sound.0.clone()), PlaybackSettings::DESPAWN));
    collision_event_reader.clear();
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

const MAX_CONTROL: f32 = 300.0;
const INPUT_CHANGE: f32 = 10.0;
fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Single<(&mut LinearVelocity, &mut Player)>,
    time: Res<Time>,
) {
    let (mut linear, mut player) = query.into_inner();
    let delta_secs = time.delta_secs();
    let mut direction = Vec2::ZERO;
    let mut rotation = 0.0;
    {
        if player.can_jump && keyboard_input.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
            if linear.y < MAX_CONTROL {
                direction.y += INPUT_CHANGE * 50.0;
            }
            player.can_jump = false;
            player.is_in_air = true;
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
    }
    let move_delta = 100.0 * direction * time.delta_secs();
    if move_delta != Vec2::ZERO {
        linear.0 += move_delta;
    }
}

fn player_fast_falling(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Single<(&mut Transform, &LinearVelocity, &mut Player)>,
    time: Res<Time>,
) {
    let (mut transform, linear, player) = query.into_inner();
    let delta = time.delta_secs();
    if player.is_in_air && linear.y < 0.0 {
        transform.translation.y -= (GRAVITY / 2.0) * delta * delta
    }
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
        Player {
            ..Default::default()
        },
        RigidBody::Dynamic,
        LockedAxes::ROTATION_LOCKED,
        Collider::rectangle(100.0, 100.0),
        Mesh2d(meshes.add(Rectangle::new(100.0, 100.0))),
        MeshMaterial2d(materials.add(Color::BLACK)),
        Transform::from_xyz(-300.0, 0.0, 0.0),
    ));
}
