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

// const SCALE: f32 = 0.3;
const SCALE: f32 = 1.0;
// how many pixels
// with damping applied, values are not accurate anymore
const JUMP_HEIGHT: f32 = 8.0;

// how long does it take to reach the maximum height of a jump?
// note: if "jump_power_coefficient" is not a multiple of "g" the maximum height is reached between frames
// second
const TIME_TO_APEX: f32 = 0.4;

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
            max_speed: 30.0,
            damping: 2.0,
            friction: 1.0,
            impulse_exponent: 4.0,
            impulse_coefficient: 300.0,
            jump_power_coefficient: 0.0,
            jump_brake_coefficient: 0.02,
            start_fall_before_peak: 10.0,
            start_of_fall_range: 10.0,
            start_of_fall_gravity_boost: 30.0,
            fall_boost_coefficient: 1.06,
            stood_on_time_coefficient: 10.0,
            uphill_move_exponent: 0.5,
            downhill_brake_exponent: 1.0,
        })
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut rapier_config: ResMut<RapierConfiguration>,
    mut player_movement_settings: ResMut<PlayerMovementSettings>,
) {
    // what is the gravity that would allow jumping to a given height?
    rapier_config.gravity.y = -(2.0 * JUMP_HEIGHT) / TIME_TO_APEX.powf(2.0);

    // what is the initial jump velocity?
    player_movement_settings.jump_power_coefficient =
        (2.0 * rapier_config.gravity.y.abs() * JUMP_HEIGHT).sqrt();

    let mut camera = OrthographicCameraBundle::new_2d();
    let zoom = 20.0;
    camera.transform.scale.x /= zoom;
    camera.transform.scale.y /= zoom;
    camera.transform.translation.x += 7.5;
    camera.transform.translation.y += 9.0;
    commands.spawn_bundle(camera);
}

#[derive(Debug)]
pub struct PlayerMovementSettings {
    // metre
    pub jump_height: f32,
    // second
    pub time_to_apex: f32,
    pub max_speed: f32,
    pub damping: f32,
    pub friction: f32,
    pub impulse_exponent: f32,
    pub impulse_coefficient: f32,
    pub jump_power_coefficient: f32,
    pub jump_brake_coefficient: f32,
    pub start_fall_before_peak: f32,
    pub start_of_fall_range: f32,
    pub start_of_fall_gravity_boost: f32,
    pub fall_boost_coefficient: f32,
    pub stood_on_time_coefficient: f32,
    pub uphill_move_exponent: f32,
    pub downhill_brake_exponent: f32,
}
