# robocomp_rapier3d

[Rapier 3D][bevy-rapier-github] physics integration for [robocomp](https://github.com/nilaysavant/robocomp) — the physics-agnostic core crate for robot/rigid-body composition in [Bevy][bevy-web].

This crate re-exports `robocomp`, `rc`, and `rd`, so you typically depend on `robocomp_rapier3d` only and get the full robocomp API alongside the [Rapier][bevy-rapier-github] backend.

## Plugins

| Plugin | Purpose |
| ------ | ------- |
| [`RobocompRapierPlugin`] | Bundles [`RobocompPlugin`] with the [Rapier][bevy-rapier-github] pre-processor that spawns colliders and joints from `Rc*` scene markers. |
| [`RobocompRapierControllerPlugin`] | Optional keyboard motor control for revolute and prismatic joints (used by the `*_ctrl_*` examples). |

The [Rapier][bevy-rapier-github] physics plugin must be added **alongside** the robocomp backend — this crate does not bundle [`bevy_rapier3d`][bevy-rapier-github].

## Quick Start

```toml
[dependencies]
robocomp_rapier3d = "0.1"
```

```rust,ignore
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use robocomp_rapier3d::{RobocompRapierPlugin, rc::RcSceneRoot};

app.add_plugins((
    RapierPhysicsPlugin::<NoUserData>::default(),
    RobocompRapierPlugin,
));
```

Controller-driven examples also require `RobocompRapierControllerPlugin`:

```rust,ignore
use robocomp_rapier3d::{RobocompRapierControllerPlugin, RobocompRapierPlugin};

app.add_plugins((
    RapierPhysicsPlugin::<NoUserData>::default(),
    RobocompRapierPlugin,
    RobocompRapierControllerPlugin,
));
```

From the workspace root (assets live at the repo root):

```bash
cargo run -p robocomp_rapier3d --example simple_rigid_bodies_rapier3d
```

## Examples

| Example | Description | Run |
| ------- | ----------- | --- |
| [`simple_rigid_bodies_rapier3d`](examples/simple_rigid_bodies_rapier3d.rs) | A simple rigid bodies example with a fixed table and cube(s) that fall onto it. | `cargo run -p robocomp_rapier3d --example simple_rigid_bodies_rapier3d` |
| [`revolute_rapier3d`](examples/revolute_rapier3d.rs) | Example demonstrating a simple robot with a revolute joint between two cubes. | `cargo run -p robocomp_rapier3d --example revolute_rapier3d` |
| [`revolute_skein_rapier3d`](examples/revolute_skein_rapier3d.rs) | Example demonstrating a scene imported from [Blender][blender-web] using [Skein][skein-github], with a revolute joint between two cubes. | `cargo run -p robocomp_rapier3d --example revolute_skein_rapier3d` |
| [`prismatic_ctrl_rapier3d`](examples/prismatic_ctrl_rapier3d.rs) | Example demonstrating a robot with a prismatic joint (w/ limits) between two cubes; control via W/S + ShiftLeft. | `cargo run -p robocomp_rapier3d --example prismatic_ctrl_rapier3d` |
| [`multi_joints_ctrl_rapier3d`](examples/multi_joints_ctrl_rapier3d.rs) | Robot chain with interleaved revolute and prismatic joints (5 links); W/S + Shift to drive, Arrow Up/Down to cycle active joint. | `cargo run -p robocomp_rapier3d --example multi_joints_ctrl_rapier3d` |

## Backend Specifics and Caveats

Robocomp is **physics-agnostic** — switching backends requires minimal code and scene changes, but **motor tuning and simulation feel are not guaranteed to match** out of the box. The same `Rd*` / `Rc*` components are mapped differently per backend. If porting from another physics backend, consider the differences below.

| Topic | Behavior |
| ----- | -------- |
| **`RdMotorModel::SpringDamper`** | No native [Rapier][bevy-rapier-github] equivalent. Mapped to `MotorModel::AccelerationBased` with converted stiffness/damping: `stiffness = (2π·f)²`, `damping = 2·ζ·(2π·f)`. Behavior differs from [Avian][avian-github]'s implicit spring-damper integrator. |
| **`RdMotorModel::AccelerationBased`** | Maps to [Rapier][bevy-rapier-github] `AccelerationBased`. |
| **`RdMotorModel::ForceBased`** | Maps to [Rapier][bevy-rapier-github] `ForceBased`. |
| **Velocity motor damping** | [Rapier][bevy-rapier-github] `set_motor_velocity(target, factor)` uses the model's **damping coefficient** as the velocity factor. Not the same semantics as [Avian][avian-github] velocity motors. |
| **Recommended motor models** | Examples use `AccelerationBased { stiffness, damping }` for position-controlled prismatic joints. The [Avian prismatic example](../robocomp_avian3d/examples/prismatic_ctrl_avian3d.rs)'s `SpringDamper` is roughly equivalent to `{ stiffness: 5000., damping: 650. }` on Rapier. |
| **`RcRigidBody::Fixed`** | Maps to [Rapier][bevy-rapier-github] `RigidBody::Fixed`. |
| **`RcCcd`** | Supported — inserts [Rapier][bevy-rapier-github] `Ccd::enabled()`. |
| **`RcDisableSleep`** | Supported — inserts `Sleeping::disabled()`. |
| **Spherical joint motors** | Spawned in the pre-processor, but controller motor control for spherical joints is not implemented yet. |
| **Controller timing** | Motor control runs in `FixedUpdate`, before `PhysicsSet::SyncBackend`. |

## Bevy Compatibility

| Bevy | [bevy_rapier3d][bevy-rapier-github] | robocomp_rapier3d |
| ---- | ------------------------------------- | ----------------- |
| **0.18** | 0.34 | 0.1 |

[bevy-web]: https://bevy.org/
[bevy-rapier-github]: https://github.com/dimforge/bevy_rapier
[avian-github]: https://github.com/avianphysics/avian
[blender-web]: https://www.blender.org/
[skein-github]: https://github.com/rust-adventure/skein
[`RobocompPlugin`]: https://github.com/nilaysavant/robocomp/blob/main/src/plugin.rs
[`RobocompRapierPlugin`]: src/rapier_plugin.rs
[`RobocompRapierControllerPlugin`]: src/controller_plugin.rs
