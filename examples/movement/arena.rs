use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_arena);
    }
}

fn setup_arena(mut commands: Commands) {
    for (x, y, half_width, half_height, rotation) in [
        (0.0, -80.0, 1000.0, 20.0, 0.0),
        (120.0, 40.0, 40.0, 10.0, 0.0),
        (-120.0, 0.0, 40.0, 10.0, 0.0),
        (450.0, 50.0, 200.0, 20.0, std::f32::consts::FRAC_PI_4),
        (-600.0, 50.0, 200.0, 20.0, std::f32::consts::FRAC_PI_2),
    ] {
        commands.spawn((
            RigidBody::Fixed,
            TransformBundle::from(
                Transform::from_xyz(x, y, 0.0).with_rotation(Quat::from_rotation_z(rotation)),
            ),
            Collider::cuboid(half_width, half_height),
        ));
    }
}
