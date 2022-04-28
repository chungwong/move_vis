use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use nalgebra::Vector2;

use move_vis::TrackMovement;

use crate::PlayerMovementSettings;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_player)
            .add_system(control_player);
    }
}

#[derive(Debug)]
enum JumpStatus {
    CanJump,
    InitiateJump,
    GoingUp,
    StoppingUp,
    GoingDown,
}

fn setup_player(mut commands: Commands, player_movement_settings: Res<PlayerMovementSettings>) {
    commands
        .spawn()
        .insert(RigidBody::Dynamic)
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Damping {
            linear_damping: player_movement_settings.damping,
            ..default()
        })
        .insert(Friction::new(1.5))
        .insert(Collider::cuboid(0.25, 1.0))
        .insert(GravityScale::default())
        .insert(MassProperties::default())
        .insert(ExternalImpulse::default())
        .insert(Velocity::zero())
        .insert(Transform::default())
        .insert(TrackMovement)
        .insert(PlayerControl {
            mid_jump: false,
            last_stood_on: Vec2::new(0.0, 1.0),
            stood_on_potential: 0.0,
        });
}

#[derive(Component)]
pub struct PlayerControl {
    mid_jump: bool,
    last_stood_on: Vec2,
    stood_on_potential: f32,
}

// logic is copied from https://github.com/idanarye/testing-physics-based-movement
fn control_player(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut query: Query<(
        Entity,
        &mut Velocity,
        &mut GravityScale,
        &mut ExternalImpulse,
        &mut PlayerControl,
    )>,
    player_movement_settings: Res<PlayerMovementSettings>,
    rapier_context: Res<RapierContext>,
) {
    let is_jumping = input.pressed(KeyCode::Space);
    let mut target_speed: f32 = 0.0;

    if input.pressed(KeyCode::A) || input.pressed(KeyCode::Left) {
        target_speed -= 1.0;
    }

    if input.pressed(KeyCode::D) || input.pressed(KeyCode::Right) {
        target_speed += 1.0;
    }

    for (
        player_entity,
        mut velocity,
        mut gravity_scale,
        mut external_impulse,
        mut player_control,
    ) in query.iter_mut()
    {
        let player_handle = rapier_context.entity2body().get(&player_entity);

        let standing_on = rapier_context
            .contacts_with(player_entity)
            .filter(|contact| contact.has_any_active_contacts())
            .flat_map(|contact| {
                contact.raw.manifolds.iter().filter_map(|contact_manifold| {
                    if contact_manifold.data.rigid_body1.as_ref() == player_handle {
                        Some(-contact_manifold.data.normal)
                    } else if contact_manifold.data.rigid_body2.as_ref() == player_handle {
                        Some(contact_manifold.data.normal)
                    } else {
                        None
                    }
                })
            })
            .max_by_key(|normal| float_ord::FloatOrd(normal.dot(&Vector2::new(0.0, 1.0))));

        // determine jump status of player
        let jump_status = (|| {
            if let Some(standing_on) = standing_on {
                player_control.last_stood_on = standing_on.into();
                player_control.stood_on_potential = 1.0;
                if 0.0 < standing_on.dot(&Vector2::new(0.0, 1.0)) {
                    if is_jumping {
                        return JumpStatus::InitiateJump;
                    }
                    return JumpStatus::CanJump;
                }
            }

            player_control.stood_on_potential = (player_control.stood_on_potential
                - time.delta().as_secs_f32() * player_movement_settings.stood_on_time_coefficient)
                .max(0.0);

            if 0.0 <= velocity.linvel.y {
                if is_jumping && player_control.mid_jump {
                    JumpStatus::GoingUp
                } else {
                    JumpStatus::StoppingUp
                }
            } else {
                JumpStatus::GoingDown
            }
        })();

        match jump_status {
            JumpStatus::GoingDown => gravity_scale.0 = 5.0,
            _ => gravity_scale.0 = 1.0,
        };

        let mut jump_impulse = Vec2::new(0.0, 0.0);

        match jump_status {
            JumpStatus::CanJump => {
                player_control.mid_jump = false;
            }
            JumpStatus::InitiateJump => {
                player_control.mid_jump = true;
                jump_impulse =
                    Vec2::new(0.0, 1.0) * player_movement_settings.jump_power_coefficient;
            }
            JumpStatus::GoingUp => {
                player_control.mid_jump = true;
            }
            JumpStatus::StoppingUp => {
                player_control.mid_jump = false;
                velocity.linvel.y *= player_movement_settings
                    .jump_brake_coefficient
                    .powf(time.delta().as_secs_f32());
                if velocity.linvel.y < player_movement_settings.start_fall_before_peak {
                    velocity.linvel.y -= player_movement_settings.start_of_fall_gravity_boost
                        * time.delta().as_secs_f32();
                }
            }
            JumpStatus::GoingDown => {
                if -player_movement_settings.start_of_fall_range < velocity.linvel.y {
                    // reminder: linvel.y is negative here
                    velocity.linvel.y -= player_movement_settings.start_of_fall_gravity_boost
                        * time.delta().as_secs_f32();
                } else {
                    velocity.linvel.y *= player_movement_settings
                        .fall_boost_coefficient
                        .powf(time.delta().as_secs_f32());
                }
                player_control.mid_jump = false;
            }
        }

        let mut up_now = Vec2::new(0.0, 1.0);
        up_now = (1.0 - player_control.stood_on_potential) * up_now
            + player_control.stood_on_potential * player_control.last_stood_on;

        let movement_vector = bevy::math::Mat2::from_angle(-std::f32::consts::FRAC_PI_2) * up_now;

        let current_speed =
            velocity.linvel.dot(movement_vector) / player_movement_settings.max_speed;

        if (0.0 < target_speed && target_speed <= current_speed)
            || (target_speed < 0.0 && current_speed <= target_speed)
        {
            continue;
        }

        let impulse = target_speed - current_speed;
        let impulse = if 1.0 < impulse.abs() {
            impulse.signum()
        } else {
            impulse.signum()
                * impulse
                    .abs()
                    .powf(player_movement_settings.impulse_exponent)
        };
        let mut impulse = movement_vector
            * time.delta().as_secs_f32()
            * player_movement_settings.impulse_coefficient
            * impulse;

        let uphill = impulse.normalize().dot(Vec2::new(0.0, 1.0));
        if 0.01 <= uphill {
            let efficiency = if target_speed.signum() as i32 == current_speed.signum() as i32 {
                player_movement_settings.uphill_move_exponent
            } else {
                player_movement_settings.downhill_brake_exponent
            };
            impulse *= 1.0 - uphill.powf(efficiency);
        }
        external_impulse.impulse = impulse + jump_impulse;
    }
}
