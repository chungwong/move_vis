use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_rapier2d::prelude::*;
use move_vis::MoveVisPlugin;

use arena::ArenaPlugin;
use player::PlayerPlugin;
use ui::UiPlugin;

mod arena;
mod player;
mod ui;

// pub const SCALE: f32 = 100.0;
// pub const SCALE: f32 = 10.0;
pub const SCALE: f32 = 1.0;
// how many pixels
// with damping applied, values are not accurate anymore
const JUMP_HEIGHT: f32 = 4.0;

// how long does it take to reach the maximum height of a jump?
// note: if "jump_power_coefficient" is not a multiple of "g" the maximum height is reached between frames
// second
const TIME_TO_APEX: f32 = 0.4;

const DEFAULT_GRAVITY_SCALE: f32 = 5.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(SCALE))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(ArenaPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(UiPlugin)
        .add_plugin(MoveVisPlugin)
        .insert_resource(PlayerMovementSettings {
            jump_height: JUMP_HEIGHT,
            time_to_apex: TIME_TO_APEX,
            run_speed: 500.0,
            dash_speed: 10000.0,
            // jump_impulse: 20000.0,
            jump_power_coefficient: 20000.0,
            coyote_time_ms: 100,
            slide_factor: 60.0,
            fall_factor: 100.0,
            jump_break_factor: 200.0,
            gravity_scale: DEFAULT_GRAVITY_SCALE,
        })
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut rapier_config: ResMut<RapierConfiguration>,
    mut player_movement_settings: ResMut<PlayerMovementSettings>,
) {
    set_gravity(&mut rapier_config, &*player_movement_settings);
    set_jump_power_coefficient(&rapier_config, &mut *player_movement_settings);

    let camera = OrthographicCameraBundle::new_2d();
    commands.spawn_bundle(camera);
}

/// what is the gravity that would allow jumping to a given height?
fn set_gravity(
    rapier_config: &mut ResMut<RapierConfiguration>,
    player_movement_settings: &PlayerMovementSettings,
) {
    rapier_config.gravity.y = -(2.0 * player_movement_settings.jump_height)
        / player_movement_settings.time_to_apex.powf(2.0);
}

/// what is the initial jump velocity?
/// 50 is a multiplier.  Say the expected value of jump_power_coefficient is 20,000 and
/// (2.0 * rapier_config.gravity.y.abs() * JUMP_HEIGHT).sqrt() gives 400.0
/// It is necessary to multiply 50.0 to reach to 20,000
fn set_jump_power_coefficient(
    rapier_config: &ResMut<RapierConfiguration>,
    player_movement_settings: &mut PlayerMovementSettings,
) {
    player_movement_settings.jump_power_coefficient =
        (2.0 * rapier_config.gravity.y.abs() * player_movement_settings.jump_height).sqrt();
    player_movement_settings.jump_power_coefficient *= 50.0 / SCALE.powf(2.0);
}

#[derive(Default)]
pub struct PlayerMovementSettings {
    // metre
    pub jump_height: f32,
    // second
    pub time_to_apex: f32,
    pub run_speed: f32,
    pub dash_speed: f32,
    // pub jump_impulse: f32,
    pub jump_power_coefficient: f32,
    pub coyote_time_ms: u64,
    // pub jump_power_coefficient: f32,
    pub slide_factor: f32,
    pub fall_factor: f32,
    pub jump_break_factor: f32,
    pub gravity_scale: f32,
}
