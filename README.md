# robocomp

[Bevy][bevy-web] Plugin for **Robot/Rigid-Body Composition** using [URDF][urdf-wiki] inspired physics agnostic components. For use with editors like [Blender][blender-web] (via [Skein][skein-github]) etc.

## Crates

| Crate                                           | Description                                                                                       |
| ----------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| [`robocomp`](Cargo.toml)                        | Physics-agnostic core: `Rd`/`Rc` components, [`RobocompPlugin`]                                   |
| [`robocomp_rapier3d`](crates/robocomp_rapier3d) | Rapier 3D backend: [`RobocompRapierPlugin`], [`RobocompRapierControllerPlugin`] — re-exports core |
| [`robocomp_avian3d`](crates/robocomp_avian3d)   | Avian 3D backend: [`RobocompAvianPlugin`], [`RobocompAvianControllerPlugin`] — re-exports core    |

For Rapier 3D simulations, depend on `robocomp_rapier3d`:

```toml
[dependencies]
robocomp_rapier3d = "0.1"
```

```rust
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use robocomp_rapier3d::{RobocompRapierPlugin, rc::RcSceneRoot};

app.add_plugins((
    RapierPhysicsPlugin::<NoUserData>::default(),
    RobocompRapierPlugin,
));
```

For Avian 3D simulations, depend on `robocomp_avian3d`:

```toml
[dependencies]
robocomp_avian3d = "0.1"
```

```rust
use bevy::prelude::*;
use avian3d::prelude::*;
use robocomp_avian3d::{RobocompAvianPlugin, rc::RcSceneRoot};

app.add_plugins((
    PhysicsPlugins::default(),
    RobocompAvianPlugin,
));
```

For a custom physics backend, depend on `robocomp` only and implement your own pre-processor.

## Workspace

```bash
# Run all tests
cargo test --workspace

# Run an example (assets live at workspace root; names: <example>[_skein]_<backend>)
cargo run -p robocomp_rapier3d --example revolute_rapier3d
cargo run -p robocomp_rapier3d --example revolute_skein_rapier3d
cargo run -p robocomp_avian3d --example revolute_avian3d
cargo run -p robocomp_avian3d --example revolute_skein_avian3d
```

## Features

- **Standardized Blender-Bevy interop**: Provides standard set of `Components` which can be set in _editors_ like Blender via plugins like [Skein][skein-github]. On exporting to `gltf/glb` and import into Bevy, they get converted into proper physics bodies with joints!
- **Easier Prefab Creation**: All you have to create are required gameplay and logic components which can be set on your gameplay objects for prefabs. No need to do re-invent the wheel of creating custom physics components every time!
- **Physics Agnostic Simulation**: Allows _swapping_ the underlying physics engine with _almost_ **NO changes** to **code** or `gltf/glb/scene`!
  - Supported Physics Engine: [`bevy_rapier`][bevy-rapier-github], [`avian`][avian-github]!
- **Easier Physics Body Queries and Control**: Robocomp provides all references to associated components of a robot at the top level and centralized. This simplifies querying and controlling the robotic contraptions in the simulation without traversing hierarchy.

[bevy-web]: https://bevy.org/
[urdf-wiki]: https://en.wikipedia.org/wiki/URDF
[skein-github]: https://github.com/rust-adventure/skein
[blender-web]: https://www.blender.org/
[bevy-rapier-github]: https://github.com/dimforge/bevy_rapier
[avian-github]: https://github.com/avianphysics/avian
[`RobocompPlugin`]: src/plugin.rs
[`RobocompRapierPlugin`]: crates/robocomp_rapier3d/src/rapier_plugin.rs
[`RobocompRapierControllerPlugin`]: crates/robocomp_rapier3d/src/controller_plugin.rs
[`RobocompAvianPlugin`]: crates/robocomp_avian3d/src/avian_plugin.rs
[`RobocompAvianControllerPlugin`]: crates/robocomp_avian3d/src/controller_plugin.rs
