use std::ops::RangeInclusive;
use std::time::Duration;

use bevy::prelude::*;
use bevy_egui::{
    egui::{
        self,
        plot::{Legend, Line, Plot, Value, Values},
    },
    EguiContext,
};

#[cfg(feature = "bevy_rapier")]
use bevy_rapier2d::prelude::Velocity;

#[cfg(feature = "heron")]
use heron::prelude::Velocity;

struct MoveVisConfig {
    track_duration: Duration,
}

impl Default for MoveVisConfig {
    fn default() -> Self {
        Self {
            track_duration: Duration::from_secs(2),
        }
    }
}

#[derive(Component)]
pub struct TrackMovement;

#[derive(Component, Debug, Default)]
pub struct History {
    pub velocity: Vec<Vec3>,
    pub distance: Vec<Vec3>,
}

#[derive(Component, Debug)]
pub struct HistoryTimer(pub Timer);

pub struct MoveVisPlugin;

impl Plugin for MoveVisPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MoveVisConfig>()
            .add_system(setup)
            .add_system(record_history)
            .add_system(plot);
    }
}

fn setup(
    mut cmd: Commands,
    move_vis_config: Res<MoveVisConfig>,
    query: Query<Entity, Added<TrackMovement>>,
) {
    for entity in query.iter() {
        let mut timer = Timer::new(move_vis_config.track_duration, false);
        timer.pause();

        cmd.entity(entity)
            .insert(History::default())
            .insert(HistoryTimer(timer));
    }
}

fn plot_distance(ui: &mut egui::Ui, history: &History) {
    let horizontal_distance = Line::new(Values::from_values_iter(
        history
            .distance
            .iter()
            .enumerate()
            .map(|(x, &v)| Value::new(x as f64, v.x as f64)),
    ))
    .color(egui::Color32::from_rgb(235, 171, 52))
    .name("Horizonal Distance");

    let vertical_distance = Line::new(Values::from_values_iter(
        history
            .distance
            .iter()
            .enumerate()
            .map(|(x, &v)| Value::new(x as f64, v.y as f64)),
    ))
    .color(egui::Color32::from_rgb(235, 64, 52))
    .name("Vertical Distance");

    Plot::new("Distance")
        .legend(Legend::default())
        .view_aspect(2.0)
        .show(ui, |plot_ui| {
            plot_ui.line(horizontal_distance);
            plot_ui.line(vertical_distance);
        });
}

fn plot_velocity(ui: &mut egui::Ui, history: &History) {
    let horinzontal_velocity = Line::new(Values::from_values_iter(
        history
            .velocity
            .iter()
            .enumerate()
            .map(|(x, &v)| Value::new(x as f64, v.x as f64)),
    ))
    .color(egui::Color32::from_rgb(100, 200, 100))
    .name("Horizontal Velocity");

    let vertical_velocity = Line::new(Values::from_values_iter(
        history
            .velocity
            .iter()
            .enumerate()
            .map(|(x, &v)| Value::new(x as f64, v.y as f64)),
    ))
    .color(egui::Color32::from_rgb(100, 150, 250))
    .name("Vertical Velocity");

    Plot::new("Velocity")
        .legend(Legend::default())
        .view_aspect(2.0)
        .show(ui, |plot_ui| {
            plot_ui.line(horinzontal_velocity);
            plot_ui.line(vertical_velocity);
        });
}

pub fn make_slider<'a, T: egui::emath::Numeric>(
    caption: &'a str,
    property: &'a mut T,
    range: RangeInclusive<T>,
) -> egui::Slider<'a> {
    egui::Slider::new(property, range).text(caption)
}

fn plot_ui(ui: &mut egui::Ui, mut move_vis_config: ResMut<MoveVisConfig>) {
    let mut secs = move_vis_config.track_duration.as_secs();

    ui.add(make_slider("Duration(secs)", &mut secs, 1..=20));

    move_vis_config.track_duration = Duration::from_secs(secs);
}

fn plot(
    move_vis_config: ResMut<MoveVisConfig>,
    mut egui_context: ResMut<EguiContext>,
    query: Query<&History, With<TrackMovement>>,
) {
    egui::Window::new("Movement").show(egui_context.ctx_mut(), |ui| {
        plot_ui(ui, move_vis_config);

        for history in query.iter() {
            plot_distance(ui, history);

            plot_velocity(ui, history);
        }
    });
}

fn record_history(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut query: Query<(&Velocity, &Transform, &mut History, &mut HistoryTimer), With<TrackMovement>>,
) {
    for (velocity, transform, mut history, mut history_timer) in query.iter_mut() {
        if input.pressed(KeyCode::A)
            || input.pressed(KeyCode::Left)
            || input.pressed(KeyCode::D)
            || input.pressed(KeyCode::Right)
            || input.pressed(KeyCode::Space)
        {
            if history_timer.0.paused() {
                history_timer.0.unpause();
            }

            if history_timer.0.finished() {
                history.distance.clear();
                history.velocity.clear();
                history_timer.0.reset();
            }
        }

        if !history_timer.0.finished() && !history_timer.0.paused() {
            history.distance.push(transform.translation);

            #[cfg(feature = "bevy_rapier")]
            history.velocity.push(velocity.linvel.extend(0.0));

            #[cfg(feature = "heron")]
            history.velocity.push(velocity.linear);
        }

        history_timer.0.tick(time.delta());
    }
}
