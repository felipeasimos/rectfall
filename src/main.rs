use avian2d::prelude::*;
use bevy::asset::AssetMetaCheck;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;

const GRAVITY: f32 = 1000.0;
const MAX_HORIZONTAL_CONTROL: f32 = 300.0;
const HORIZONTAL_CHANGE: f32 = 10.0;
const JUMP_BOOST: f32 = 100.0;

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
        .add_systems(Startup, (setup, spawn_floor, spawn_wall, spawn_player))
        .add_systems(Update, move_camera)
        .add_systems(Update, handle_collision)
        .add_systems(FixedPreUpdate, move_player)
        .add_systems(FixedPostUpdate, player_fast_falling)
        .run();
}

#[derive(Component)]
struct Player {
    can_jump: bool,
    started_jump_press_duration: f32,
    finished_jump_press: bool,
    is_attached_to_wall: bool,
}

impl Player {
    fn reset_jump(&mut self) {
        *self = Player {
            ..Default::default()
        };
    }
}

impl Default for Player {
    fn default() -> Player {
        Player {
            can_jump: false,
            started_jump_press_duration: 0.0,
            finished_jump_press: false,
            is_attached_to_wall: false,
        }
    }
}

#[derive(Resource)]
struct CollisionSound(Handle<AudioSource>);

fn handle_player_collision(player: &mut Player, contact_normal: Vec2) {
    let dot = contact_normal.dot(Vec2::NEG_Y);
    if dot.abs() < 0.1 {
        // wall
        println!("wall collision");
        player.is_attached_to_wall = true;
    } else if dot.abs() > 0.9 {
        // ground
        println!("ground collision");
        player.reset_jump();
        player.can_jump = true;
    }
}

fn handle_collision(
    mut commands: Commands,
    collisions: Res<Collisions>,
    mut single: Single<(Entity, &mut Player)>,
    sound: Res<CollisionSound>,
) {
    let (player_entt, mut player) = single.into_inner();
    for coll in collisions.collisions_with_entity(player_entt) {
        // ignore non-initial collisions
        if let Some(contact_data) = coll.find_deepest_contact() {
            // ignore speculative collisions
            if contact_data.penetration < 0.0 {
                continue;
            }
            let player_contact_normal = if coll.entity1 == player_entt {
                contact_data.normal1
            } else {
                contact_data.normal2
            };
            handle_player_collision(&mut *player, player_contact_normal);
        }
    }
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Single<(&mut LinearVelocity, &mut Player)>,
    time: Res<Time>,
) {
    let (mut linear, mut player) = query.into_inner();
    let delta_secs = time.delta_secs();
    let mut direction = Vec2::ZERO;
    {
        if keyboard_input.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
            if player.can_jump {
                player.can_jump = false;
                player.started_jump_press_duration = delta_secs;
                if linear.y < MAX_HORIZONTAL_CONTROL {
                    direction.y = JUMP_BOOST;
                }
            } else if !player.finished_jump_press && player.started_jump_press_duration > 0.5 {
                player.finished_jump_press = true;
            } else if player.started_jump_press_duration > 0.0 && !player.finished_jump_press {
                player.started_jump_press_duration += delta_secs;
                if linear.y < MAX_HORIZONTAL_CONTROL {
                    direction.y = JUMP_BOOST;
                }
            } else if player.is_attached_to_wall {
                player.is_attached_to_wall = false;
                player.started_jump_press_duration = delta_secs;
                player.finished_jump_press = false;
                if linear.y < MAX_HORIZONTAL_CONTROL {
                    direction.y = JUMP_BOOST;
                }
                direction.x -= JUMP_BOOST;
            }
        } else if player.started_jump_press_duration > 0.0 {
            player.finished_jump_press = true;
        }
        if keyboard_input.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
            if linear.x < MAX_HORIZONTAL_CONTROL {
                direction.x += HORIZONTAL_CHANGE;
            }
        }
        if keyboard_input.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
            if -linear.x < MAX_HORIZONTAL_CONTROL {
                direction.x -= HORIZONTAL_CHANGE;
            }
        }
    }
    let move_delta = 100.0 * direction * delta_secs;
    if move_delta != Vec2::ZERO {
        linear.0 += move_delta;
    }
}

fn player_fast_falling(
    query: Single<(&mut Transform, &LinearVelocity, &mut Player)>,
    time: Res<Time>,
) {
    let (mut transform, linear, player) = query.into_inner();
    let delta = time.delta_secs();
    if player.started_jump_press_duration > 0.0 && linear.y < 0.0 {
        transform.translation.y -= (GRAVITY / 2.0) * delta * delta
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

fn spawn_wall(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Name::new("RightWall"),
        RigidBody::Static,
        Collider::rectangle(100.0, 1000.0),
        Mesh2d(meshes.add(Rectangle::new(100.0, 1000.0))),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Transform::from_xyz(100.0, -300.0, 0.0),
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
