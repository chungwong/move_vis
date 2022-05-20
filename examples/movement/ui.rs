use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use bevy_rapier2d::prelude::*;

use move_vis::make_slider;

use crate::{set_gravity, set_jump_power_coefficient, PlayerMovementSettings};

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
) {
    egui::Window::new("Physical Properties Tweaking").show(egui_context.ctx_mut(), |ui| {
        let player_movement_settings = &mut *player_movement_settings;

        ui.add(make_slider(
            "Jump Height",
            &mut player_movement_settings.jump_height,
            1.0..=20.0,
        ));
        ui.add(make_slider(
            "Time To Apex",
            &mut player_movement_settings.time_to_apex,
            0.1..=1.0,
        ));
        ui.add(make_slider(
            "Run Speed",
            &mut player_movement_settings.run_speed,
            100.0..=2000.0,
        ));
        ui.add(make_slider(
            "Dash Speed",
            &mut player_movement_settings.dash_speed,
            5000.0..=20000.0,
        ));
        // ui.add(make_slider(
        //     "Jump Impulse",
        //     &mut player_movement_settings.jump_impulse,
        //     10000.0..=50000.0,
        // ));
        ui.add(make_slider(
            "Coyote Time(ms)",
            &mut player_movement_settings.coyote_time_ms,
            10..=200,
        ));
        ui.add(make_slider(
            "Slide Factor",
            &mut player_movement_settings.slide_factor,
            0.0..=-1000.0,
        ));
        ui.add(make_slider(
            "Fall Factor",
            &mut player_movement_settings.fall_factor,
            50.0..=200.0,
        ));
        ui.add(make_slider(
            "Jump Break Factor",
            &mut player_movement_settings.jump_break_factor,
            100.0..=400.0,
        ));
        ui.add(make_slider(
            "Gravity Scale",
            &mut player_movement_settings.gravity_scale,
            1.0..=20.0,
        ));

        set_gravity(&mut rapier_config, player_movement_settings);
        set_jump_power_coefficient(&rapier_config, &mut *player_movement_settings);
    });
}
