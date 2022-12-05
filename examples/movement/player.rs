use std::convert::TryFrom;
use std::time::Duration;

use bevy::{prelude::*, utils::Instant};
use bevy_rapier2d::prelude::*;

use move_vis::TrackMovement;

use crate::PlayerMovementSettings;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Direction {
    Left = -1,
    Neutral = 0,
    Right = 1,
}

impl Direction {
    fn to_f32(self) -> f32 {
        match self {
            Self::Left => -1.0,
            Self::Right => 1.0,
            Self::Neutral => 0.0,
        }
    }
}

impl Default for Direction {
    fn default() -> Self {
        Direction::Neutral
    }
}

impl TryFrom<f32> for Direction {
    type Error = ();

    fn try_from(v: f32) -> Result<Self, Self::Error> {
        match v as isize {
            -1 => Ok(Direction::Left),
            1 => Ok(Direction::Right),
            0 => Ok(Direction::Neutral),
            _ => Err(()),
        }
    }
}

#[derive(Resource)]
struct DashInput {
    input_timer: Timer,
    direction: Direction,
}

impl Default for DashInput {
    fn default() -> Self {
        Self {
            input_timer: Timer::new(Duration::from_millis(200), TimerMode::Once),
            direction: Direction::Neutral,
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DashInput>()
            .add_startup_system(setup_player)
            .add_system(check_standing)
            .add_system(dash.after(check_standing))
            .add_system(run.after(check_standing))
            .add_system(jump.after(check_standing));
    }
}

fn check_standing(
    time: Res<Time>,
    mut query: Query<(Entity, &mut PlayerControl)>,
    rapier_context: Res<RapierContext>,
) {
    for (player_entity, mut player_control) in query.iter_mut() {
        if get_standing_normal(&rapier_context, &player_entity).is_some() {
            player_control.last_stood_time = time.last_update();
        }
    }
}

fn dash(
    time: Res<Time>,
    mut dash_input: Local<DashInput>,
    input: Res<Input<KeyCode>>,
    mut query: Query<(Entity, &mut Velocity, &mut GravityScale, &mut PlayerControl)>,
    rapier_context: Res<RapierContext>,
    player_movement_settings: Res<PlayerMovementSettings>,
) {
    for (player_entity, mut velocity, mut gravity_scale, mut player_control) in query.iter_mut() {
        let dir = if input.just_pressed(KeyCode::A) || input.just_pressed(KeyCode::Left) {
            Direction::Left
        } else if input.just_pressed(KeyCode::D) || input.just_pressed(KeyCode::Right) {
            Direction::Right
        } else {
            Direction::Neutral
        };

        let standing_normal = get_standing_normal(&rapier_context, &player_entity);

        if let Some(normal) = standing_normal {
            // if it is standing on something and LEFT or Right is pressed
            if normal == Vec2::Y && matches!(&dir, Direction::Left | Direction::Right) {
                if dash_input.input_timer.finished() {
                    // store the input, to check if the consecutive inputs are in the same dir
                    dash_input.direction = dir;

                    // reset the timer
                    dash_input.input_timer.reset();

                    *gravity_scale = GravityScale(player_movement_settings.gravity_scale);
                } else if !dash_input.input_timer.finished() && dir == dash_input.direction {
                    velocity.linvel = get_run_velocity(
                        &velocity.linvel,
                        dir.to_f32() * player_movement_settings.dash_speed,
                        time.delta_seconds(),
                    );

                    player_control.dashing = true;

                    *gravity_scale = GravityScale(0.0);
                } else {
                    *gravity_scale = GravityScale(player_movement_settings.gravity_scale);
                }
            }
        }

        if dash_input.input_timer.finished() && player_control.dashing {
            player_control.dashing = false;

            // if dash is finished, reset gravity scale otherwise, when doing a dash jump
            // and holding space, the player will fly
            *gravity_scale = GravityScale(player_movement_settings.gravity_scale);
        }
    }
    if !dash_input.input_timer.finished() {
        dash_input.input_timer.tick(time.delta());
    }
}

#[derive(Debug)]
enum JumpStatus {
    CanJump,
    InitiateJump,
    InitiateWallJump,
    GoingUp,
    StoppingUp,
    GoingDown,
    WallSliding,
}

fn setup_player(mut commands: Commands, player_movement_settings: Res<PlayerMovementSettings>) {
    commands.spawn((
        RigidBody::Dynamic,
        Damping {
            linear_damping: 10.0,
            ..default()
        },
        LockedAxes::ROTATION_LOCKED,
        Collider::cuboid(5.0, 5.0),
        GravityScale(player_movement_settings.gravity_scale),
        ExternalImpulse::default(),
        ExternalForce::default(),
        ColliderMassProperties::Density(1.0),
        Velocity::zero(),
        TrackMovement,
        PlayerControl::new(),
    ));
}

#[derive(Component, Debug, Default)]
pub struct PlayerControl {
    dashing: bool,
    rising: bool,
    jumping: bool,
    wall_sliding: bool,
    wall_jumping: bool,
    last_stood_normal: Vec2,
    last_stood_time: Option<Instant>,
}

impl PlayerControl {
    fn new() -> Self {
        Self {
            last_stood_normal: Vec2::Y,
            ..default()
        }
    }
}

fn get_run_velocity(velocity: &Vec2, speed: f32, time_delta: f32) -> Vec2 {
    let wall_jump_lerp = 10.;
    velocity.lerp(Vec2::new(speed, velocity.y), wall_jump_lerp * time_delta)
}

/// https://en.wikipedia.org/wiki/Normal_(geometry)
fn get_standing_normal(
    rapier_context: &Res<RapierContext>,
    player_entity: &Entity,
) -> Option<Vec2> {
    let mut standing_normal = rapier_context
        .contacts_with(*player_entity)
        .filter(|contact| contact.has_any_active_contacts())
        .flat_map(|contact| {
            contact
                .manifolds()
                .filter_map(|contact_manifold| {
                    if contact.collider1() == *player_entity {
                        Some(-contact_manifold.normal())
                    } else if contact.collider2() == *player_entity {
                        Some(contact_manifold.normal())
                    } else {
                        None
                    }
                })
                .max_by_key(|normal| float_ord::FloatOrd(normal.dot(Vec2::Y)))
        })
        .max_by_key(|normal| float_ord::FloatOrd(normal.dot(Vec2::Y)));

    if let Some(mut normal) = standing_normal {
        // truncate float number with a `long tail`
        if normal.y < 0.0001 {
            normal.y = 0.0;
        }

        standing_normal = Some(normal);
    }

    standing_normal
}

fn jump(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut query: Query<(Entity, &mut Velocity, &mut PlayerControl)>,
    player_movement_settings: Res<PlayerMovementSettings>,
    rapier_context: Res<RapierContext>,
    rapier_config: Res<RapierConfiguration>,
) {
    let pressed_jump = input.pressed(KeyCode::Space);

    // let jump_impulse = 10000.0; // SCALE = 1.0
    // let jump_impulse = 100.0;  // SCALE = 10.0
    // let jump_impulse = 1.0;   // SCALE = 100.0
    // let jump_impulse = player_movement_settings.jump_impulse / SCALE.powf(2.0);

    for (player_entity, mut velocity, mut player_control) in query.iter_mut() {
        // find a normal of the standing ground where the player stands on
        let mut standing_normal = get_standing_normal(&rapier_context, &player_entity);

        // coyote time if
        // 1. the Y normal is none(falling)
        // 2. player pressed_jump
        // 3. the player is not currently jumping
        if standing_normal.is_none() && pressed_jump && !player_control.jumping {
            let duration = Duration::from_millis(player_movement_settings.coyote_time_ms);
            match (player_control.last_stood_time, time.last_update()) {
                (Some(last_stood_time), Some(last_update))
                    if last_update - last_stood_time <= duration =>
                {
                    standing_normal = Some(player_control.last_stood_normal);
                }
                _ => (),
            };
        }

        player_control.wall_sliding = false;

        if let Some(normal) = standing_normal {
            // reset player_control.wall_jumping and player_control.jumping when it is jump back on ground
            // 1. on a ground
            // 2. on wall grab
            if normal.x.abs() == 1.0 || normal.y == 1.0 {
                player_control.jumping = false;
                player_control.wall_jumping = false;
            }
        }

        // determie the jump status of the player
        let jump_status = (|| {
            if let Some(normal) = standing_normal {
                player_control.last_stood_normal = normal;

                // // wall grab and slide
                if normal.x.abs() == 1.0
                    && normal.y == 0.0
                    && (input.pressed(KeyCode::D) || input.pressed(KeyCode::A))
                {
                    if input.just_pressed(KeyCode::Space) {
                        return JumpStatus::InitiateWallJump;
                    }
                    return JumpStatus::WallSliding;
                }

                if 0.0 < normal.dot(Vec2::Y) && normal.y > 0.001 {
                    if pressed_jump {
                        return JumpStatus::InitiateJump;
                    }
                    return JumpStatus::CanJump;
                }
            }

            if 0.0 <= velocity.linvel.y {
                if pressed_jump && player_control.rising {
                    JumpStatus::GoingUp
                } else {
                    JumpStatus::StoppingUp
                }
            } else {
                JumpStatus::GoingDown
            }
        })();

        match jump_status {
            JumpStatus::CanJump => {
                player_control.rising = false;
            }
            JumpStatus::InitiateJump => {
                velocity.linvel += Vec2::Y * player_movement_settings.jump_power_coefficient;

                player_control.rising = true;

                // indicate the player is jumping, it will only be reset when it touches the ground
                // or wall grab or something similar
                player_control.jumping = true;
            }
            JumpStatus::GoingUp => {
                player_control.rising = true;
            }
            JumpStatus::StoppingUp => {
                velocity.linvel.y += rapier_config.gravity.y
                    * player_movement_settings.jump_break_factor
                    * time.delta_seconds();
                player_control.rising = false;
            }
            JumpStatus::GoingDown => {
                velocity.linvel.y += rapier_config.gravity.y
                    * player_movement_settings.fall_factor
                    * time.delta_seconds();
                player_control.rising = false;
            }
            JumpStatus::WallSliding => {
                player_control.wall_sliding = true;
                player_control.rising = false;

                // wall slide
                // velocity.linvel.y += rapier_config.gravity.y * player_movement_settings.slide_factor * time.delta_seconds();

                // wall grab
                velocity.linvel.y = 0.0;
            }
            JumpStatus::InitiateWallJump => {
                player_control.wall_jumping = true;

                velocity.linvel.x = Vec2::X.x * player_movement_settings.run_speed;
                velocity.linvel.y = Vec2::Y.y * player_movement_settings.jump_power_coefficient;
            }
        }
    }
}

fn run(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Velocity, &PlayerControl)>,
    player_movement_settings: Res<PlayerMovementSettings>,
) {
    let target_speed: f32 = if input.pressed(KeyCode::A) || input.pressed(KeyCode::Left) {
        -1.0
    } else if input.pressed(KeyCode::D) || input.pressed(KeyCode::Right) {
        1.0
    } else {
        0.0
    };

    for (mut velocity, player_control) in query.iter_mut() {
        // if wall jumping, not able to move in air
        if !player_control.wall_jumping {
            velocity.linvel = get_run_velocity(
                &velocity.linvel,
                target_speed * player_movement_settings.run_speed,
                time.delta_seconds(),
            );
        }
    }
}
