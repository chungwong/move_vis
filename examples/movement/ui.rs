use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use bevy_rapier2d::prelude::*;

use move_vis::make_slider;

use crate::player::PlayerControl;
use crate::PlayerMovementSettings;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(ui_system);
    }
}

fn ui_system(
    mut egui_context: ResMut<EguiContext>,
    mut player_movement_settings: ResMut<PlayerMovementSettings>,
    mut rapier_config: ResMut<RapierConfiguration>,
    mut player: Query<(&mut Friction, &mut Damping), With<PlayerControl>>,
) {
    egui::Window::new("Physical Properties Tweaking").show(egui_context.ctx_mut(), |ui| {
        let player_movement_settings = &mut *player_movement_settings;

        let (mut friction, mut damping) = player.single_mut();

        ui.add(make_slider(
            "Jump Height",
            &mut player_movement_settings.jump_height,
            1.0..=20.0,
        ));
        ui.add(make_slider(
            "Time To Apex",
            &mut player_movement_settings.time_to_apex,
            0.1..=5.0,
        ));
        ui.add(make_slider(
            "friction",
            &mut friction.coefficient,
            0.0..=10.0,
        ));
        ui.add(make_slider(
            "Damping",
            &mut damping.linear_damping,
            0.0..=10.0,
        ));
        ui.add(make_slider(
            "Max Speed",
            &mut player_movement_settings.max_speed,
            1.0..=100.0,
        ));
        ui.add(make_slider(
            "Impulse Exponent",
            &mut player_movement_settings.impulse_exponent,
            1.0..=10.0,
        ));
        ui.add(make_slider(
            "Impulse Coefficient",
            &mut player_movement_settings.impulse_coefficient,
            100.0..=1000.0,
        ));
        ui.add(make_slider(
            "Jump Power Coefficient",
            &mut player_movement_settings.jump_power_coefficient,
            1.0..=2000.0,
        ));
        ui.add(
            make_slider(
                "Jump Brake Coefficient",
                &mut player_movement_settings.jump_brake_coefficient,
                0.0..=0.1,
            )
            .logarithmic(true),
        );
        ui.add(make_slider(
            "Start Fall Before Peak",
            &mut player_movement_settings.start_fall_before_peak,
            0.0..=40.0,
        ));
        ui.add(make_slider(
            "Start of Fall Range",
            &mut player_movement_settings.start_of_fall_range,
            0.0..=40.0,
        ));
        ui.add(make_slider(
            "Start of Fall Gravity Boost",
            &mut player_movement_settings.start_of_fall_gravity_boost,
            0.0..=100.0,
        ));
        ui.add(
            make_slider(
                "Fall Boost Coefficient",
                &mut player_movement_settings.fall_boost_coefficient,
                1.0..=2.0,
            )
            .logarithmic(true),
        );
        ui.add(make_slider(
            "Stood On Time Coefficient",
            &mut player_movement_settings.stood_on_time_coefficient,
            1.0..=100.0,
        ));
        ui.add(make_slider(
            "Uphill Move Exponent",
            &mut player_movement_settings.uphill_move_exponent,
            0.01..=200.0,
        ));
        ui.add(make_slider(
            "Downhill Brake Exponent",
            &mut player_movement_settings.downhill_brake_exponent,
            0.01..=200.0,
        ));

        rapier_config.gravity.y = -(2.0 * player_movement_settings.jump_height)
            / player_movement_settings.time_to_apex.powf(2.0);
        player_movement_settings.jump_power_coefficient =
            (2.0 * -rapier_config.gravity.y * player_movement_settings.jump_height).sqrt();
    });
}
