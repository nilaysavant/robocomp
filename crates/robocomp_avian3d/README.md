# robocomp_avian3d

[Avian 3D][avian-github] physics integration for [robocomp](https://github.com/nilaysavant/robocomp) — the physics-agnostic core crate for robot/rigid-body composition in [Bevy][bevy-web].

This crate re-exports `robocomp`, `rc`, and `rd`, so you typically depend on `robocomp_avian3d` only and get the full robocomp API alongside the [Avian][avian-github] backend.

## Plugins

| Plugin | Purpose |
| ------ | ------- |
| [`RobocompAvianPlugin`] | Bundles [`RobocompPlugin`] with the [Avian][avian-github] pre-processor that spawns colliders and joints from `Rc*` scene markers. |
| [`RobocompAvianControllerPlugin`] | Optional keyboard motor control for revolute and prismatic joints (used by the `*_ctrl_*` examples). |

The [Avian][avian-github] physics plugin must be added **alongside** the robocomp backend — this crate does not bundle [`avian3d`][avian-github].

## Quick Start

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

Controller-driven examples also require `RobocompAvianControllerPlugin`:

```rust
use robocomp_avian3d::{RobocompAvianControllerPlugin, RobocompAvianPlugin};

app.add_plugins((
    PhysicsPlugins::default(),
    RobocompAvianPlugin,
    RobocompAvianControllerPlugin,
));
```

From the workspace root (assets live at the repo root):

```bash
cargo run -p robocomp_avian3d --example simple_rigid_bodies_avian3d
```

## Examples

| Example | Description | Run |
| ------- | ----------- | --- |
| [`simple_rigid_bodies_avian3d`](examples/simple_rigid_bodies_avian3d.rs) | A simple rigid bodies example with a fixed table and cube(s) that fall onto it. | `cargo run -p robocomp_avian3d --example simple_rigid_bodies_avian3d` |
| [`revolute_avian3d`](examples/revolute_avian3d.rs) | Example demonstrating a simple robot with a revolute joint between two cubes. | `cargo run -p robocomp_avian3d --example revolute_avian3d` |
| [`revolute_skein_avian3d`](examples/revolute_skein_avian3d.rs) | Example demonstrating a scene imported from [Blender][blender-web] using [Skein][skein-github], with a revolute joint between two cubes. | `cargo run -p robocomp_avian3d --example revolute_skein_avian3d` |
| [`prismatic_ctrl_avian3d`](examples/prismatic_ctrl_avian3d.rs) | Example demonstrating a robot with a prismatic joint (w/ limits) between two cubes; control via W/S + ShiftLeft. | `cargo run -p robocomp_avian3d --example prismatic_ctrl_avian3d` |
| [`multi_joints_ctrl_avian3d`](examples/multi_joints_ctrl_avian3d.rs) | Robot chain with interleaved revolute and prismatic joints (5 links); W/S + Shift to drive, Arrow Up/Down to cycle active joint. | `cargo run -p robocomp_avian3d --example multi_joints_ctrl_avian3d` |

## Backend Specifics and Caveats

Robocomp is **physics-agnostic** — switching backends requires minimal code and scene changes, but **motor tuning and simulation feel are not guaranteed to match** out of the box. The same `Rd*` / `Rc*` components are mapped differently per backend. If porting from another physics backend, consider the differences below.

| Topic | Behavior |
| ----- | -------- |
| **Motor models** | All three `RdMotorModel` variants map **natively** (`SpringDamper`, `ForceBased`, `AccelerationBased`). |
| **Prismatic position control** | `SpringDamper` is the recommended/stable choice (see the [prismatic example](examples/prismatic_ctrl_avian3d.rs)). |
| **Revolute continuous spin** | [Avian][avian-github] wraps position error to `[-π, π]`; use **velocity** motors for continuous rotation, not `SpringDamper` position control (see the [multi-joint example](examples/multi_joints_ctrl_avian3d.rs)). |
| **`RcRigidBody::Fixed`** | Maps to [Avian][avian-github] `RigidBody::Static` (not `Fixed`). |
| **`RcCcd`** | **Not wired yet** — `RcCcd` is ignored in the [Avian][avian-github] pre-processor. |
| **`RcDisableSleep`** | Supported — inserts `SleepingDisabled`. |
| **Spherical joint motors** | **Not wired yet** — pre-processor passes `motor_x/y/z: None` regardless of `RcSphericalJointMotor`. |
| **Spherical joint limits** | Best-effort mapping: per-axis [Rapier][bevy-rapier-github]-style limits → [Avian][avian-github] swing/twist limits. |
| **Controller timing** | Motor control runs in `FixedPostUpdate`, before `PhysicsSystems::Prepare`. |

## Bevy Compatibility

| Bevy | [avian3d][avian-github] | robocomp_avian3d |
| ---- | ----------------------- | ---------------- |
| **0.18** | 0.6.1 | 0.1 |

[bevy-web]: https://bevy.org/
[avian-github]: https://github.com/avianphysics/avian
[bevy-rapier-github]: https://github.com/dimforge/bevy_rapier
[blender-web]: https://www.blender.org/
[skein-github]: https://github.com/rust-adventure/skein
[`RobocompPlugin`]: https://github.com/nilaysavant/robocomp/blob/main/src/plugin.rs
[`RobocompAvianPlugin`]: src/avian_plugin.rs
[`RobocompAvianControllerPlugin`]: src/controller_plugin.rs
