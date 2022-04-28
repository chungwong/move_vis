# Movement Visualisation

A bevy plugin to visualise the movements of a player.

[Demo](https://chungwong.github.io/move_vis/)

```rust
use move_vis::MoveVisPlugin;

fn main() {
   App::new()
       .add_plugin(MoveVisPlugin)
       .add_system(spawn_player);
   // ...
}

fn spawn_player(mut commands: Commands){
    commands
        .spawn()
        .insert(RigidBody::Dynamic)
        ..
        .insert(TrackMovement);
}
```

For more details on usage see [Examples](https://github.com/chungwong/move_vis/tree/master/examples)
