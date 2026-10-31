---
name: Bevy 0.19 upgrade
overview: Upgrade the workspace from Bevy 0.18 to 0.19.1 and move every companion crate to the matching 0.19 release, then fix the scene/world-serialization renames, convert code-built examples to BSN, and fix any remaining compile breaks.
todos:
  - id: bump-deps
    content: Pin workspace Bevy 0.19 and matching Rapier 0.36, Avian 0.7, Skein 0.6, and inspector 0.37. Do not set rust-version or a toolchain file.
    status: completed
  - id: migrate-scene-apis
    content: Rename ScenePlugin/DynamicScene/SceneRoot to the world-serialization types in the harness, skein examples, and RcSceneRoot docs
    status: completed
  - id: examples-to-bsn
    content: Rewrite code-built examples and scene_setup helpers from children!/with_child to bsn! and spawn_scene. Leave glTF skein examples on WorldAssetRoot. Add Default only on unit markers bsn! rejects.
    status: completed
  - id: fix-compile
    content: cargo check --workspace --all-targets and fix remaining 0.19 breaks (commands, features, physics APIs)
    status: completed
  - id: update-docs
    content: Update the three README compatibility tables to Bevy 0.19
    status: completed
  - id: verify
    content: Run workspace tests and example checks for both backends
    status: completed
isProject: false
---

# Upgrade the workspace to Bevy 0.19

Stay on the **0.19** line. Bevy 0.20 is already published, and the newest Rapier, Avian, Skein, and inspector releases target 0.20. Pin the 0.19-compatible versions below (same caret style as today).

Do not set `rust-version` and do not add a `rust-toolchain` file. Bevy 0.19’s own crate declares a floor of Rust 1.95, which only rejects compilers older than that. Current stable (1.99) and any newer stable satisfy it. This workspace stays on whatever stable toolchain is installed.

## Dependency pins

All versions live in `[workspace.dependencies]` in [`Cargo.toml`](Cargo.toml). Member crates already use `workspace = true`, so they do not need their own version numbers.

- `bevy`: `0.18` → `0.19` (resolves to 0.19.1). Keep `default-features = false`.
- `bevy_rapier3d`: `0.34` → `0.36` (`0.37` is Bevy 0.20). This repo does not enable the removed `simd-stable` / `simd-nightly` features.
- `avian3d`: `0.6.1` → `0.7` (`0.8` is Bevy 0.20). [Avian’s 0.6-to-0.7 guide](https://github.com/avianphysics/avian/blob/main/migration-guides/0.6-to-0.7.md) is a Bevy bump plus picking/move-and-slide changes this repo does not use. `PhysicsSystems::Prepare` is already the name in [`crates/robocomp_avian3d/src/controller_plugin.rs`](crates/robocomp_avian3d/src/controller_plugin.rs).
- `bevy_skein`: `0.5` → `0.6` (`0.7.0-rc.1` is the 0.20 line).
- `bevy-inspector-egui`: `0.36` → `0.37` (`0.38` is Bevy 0.20). Examples already call `EguiPlugin::default()`, which is what bevy_egui 0.40 expects.
- `petgraph` stays at `0.8`. Do not bump the robocomp package version (`0.1.0`).

`Cargo.lock` is gitignored, so the bump is the manifest plus a local resolve.

## Known source changes

Bevy renamed the old scene crate to world serialization ([0.18 → 0.19 migration guide](https://bevy.org/learn/migration-guides/0-18-to-0-19/)). `bevy::scene` is now BSN, so the old imports will not compile.

- [`crates/robocomp_test_utils/src/harness.rs`](crates/robocomp_test_utils/src/harness.rs): `bevy::scene::ScenePlugin` → `bevy::world_serialization::WorldSerializationPlugin`. `DynamicScene::from_world` → `DynamicWorld` (`from_world` still exists and reads `AppTypeRegistry` from the world; switch to `from_world_with` only if the registry borrow conflicts with the existing `serialize` call).
- Skein examples [`crates/robocomp_rapier3d/examples/revolute_skein_rapier3d.rs`](crates/robocomp_rapier3d/examples/revolute_skein_rapier3d.rs) and [`crates/robocomp_avian3d/examples/revolute_skein_avian3d.rs`](crates/robocomp_avian3d/examples/revolute_skein_avian3d.rs): `SceneRoot(...)` → `WorldAssetRoot(...)`. `GltfAssetLabel::Scene(0)` stays. Do not rewrite these as `bsn!`. Bevy 0.19 has not ported glTF loading to BSN, so `WorldAssetRoot` is still how a glTF scene is spawned.
- [`src/rc/mod.rs`](src/rc/mod.rs) doc link on `RcSceneRoot` points at Bevy’s `SceneRoot`. Point it at `WorldAssetRoot`. Robocomp’s own `RcSceneRoot` name stays.

## Examples to BSN

Bevy 0.19’s `bsn!` / `Commands::spawn_scene` replaces hand-built entity trees. Convert the code-built examples, in both backends (the Rapier and Avian copies match):

- [`crates/robocomp_rapier3d/examples/helpers/scene_setup.rs`](crates/robocomp_rapier3d/examples/helpers/scene_setup.rs) and the Avian copy: camera, sun, and ground plane.
- `simple_rigid_bodies_*`, `revolute_*`, `prismatic_ctrl_*`, `multi_joints_ctrl_*`.

Shape of the rewrite:

- Hierarchy helpers return `impl Scene` built with `bsn!`, not `impl Bundle` built with `children!` / `with_child`.
- Parent scenes include those functions and nest with `Children [ ... ]`. Commas separate sibling entities; components on the same entity are listed without a comma.
- Spawn with `commands.spawn_scene(...)` (immediate). Do not use `queue_spawn_scene`. The preprocessor runs in `Update` and needs `RcSceneRoot` present after `Startup` command application. Procedural meshes are already in `Assets`, so there is no asset load to wait for.
- Keep creating `Cuboid` / `Plane3d` meshes and `StandardMaterial`s in Rust and pass the handles in with `{handle}`. BSN asset-path strings do not cover meshes built in code.
- Dynamic names stay `Name({format!(...)})`. `Transform::from_translation(...)` becomes a `Transform` field patch or `template_value(...)` if a full value will not parse.

`bsn!` only accepts a component that is `Default + Clone`, or `FromTemplate`. Do not derive `FromTemplate` on robocomp types: that pulls `bevy_scene` into the core crate and conflicts with types that already implement `Default` (`RcLink`, `RcCollider`, `RcJoint`). If the macro rejects a unit marker, add `Default` to that marker only. The ones examples insert and that lack it today are `RcSceneRoot`, `RcRobotRoot`, `RcLinkRoot`, and `RcDisableSleep`. Types that already have `Default` (`RcLink`, `RcCollider`, `RcJoint`, the motor components) need no derive change. If an enum such as `RcJointKind` is rejected, pass the full value in `{...}` rather than adding `VariantDefaults` unless the compiler requires it.

Leave [`crates/robocomp_test_utils/src/scenarios.rs`](crates/robocomp_test_utils/src/scenarios.rs) on `children!`. Those are tests, not examples, and they should keep spawning during the existing headless update loop.

Dev-dependencies already list `ui` separately from `3d`, so the 0.19 change that stops `3d` from implying `ui` does not apply. Audio is unused.

## Compile-driven fixes

After the pins and renames, `cargo check --workspace --all-targets` and fix whatever the compiler reports. Expected hot spots, in order:

- [`src/cleanup_manager.rs`](src/cleanup_manager.rs): `EntityCommands::queue_handled` closures return `Result`. In 0.19 `Command<Result>` became `Command` with an associated `Out`. Update the call only if the blanket impl no longer accepts those closures. `despawn`, `despawn_related::<Children>`, and `remove_parent_in_place` are not in the migration guide.
- Feature flags on the core crate (`bevy_render`, `bevy_log`, dev `2d`/`3d`/`ui`/`bevy_dev_tools`). If `WorldSerializationPlugin` or mesh/transform types disappear behind a renamed feature, add that feature. Do not turn default features back on.
- Rapier schedule and context APIs used today (`PhysicsSet::SyncBackend`, `WriteRapierContext::single_mut`, `Collider::from_bevy_mesh`) are still the 0.36 API. Change them only if 0.36 renamed a symbol this code calls.
- Avian collider constructors (`convex_hull_from_mesh`, `ColliderDensity`, `SleepingDisabled`) and joint motor fields. The 0.7 guide does not list these; fix only on compile error.
- Observers `On<Add, Collider>` in the backend tests. That generic form is still 0.19; the observer-generic removal is a 0.20 change.
- Example helpers [`scene_setup.rs`](crates/robocomp_rapier3d/examples/helpers/scene_setup.rs) (`FpsOverlayPlugin`, `GlobalAmbientLight`, `EguiPlugin::default()`, `WorldInspectorPlugin`). No documented break; fix if `check --examples` fails. `SkeinAppExt::insert_skein_preset` in [`src/presets_plugin.rs`](src/presets_plugin.rs) should keep its signature on Skein 0.6.

Resources-as-components does not require a type split here: nothing derives both `Component` and `Resource`. Integration tests assert that a serialized world **contains** a type path, so extra resource entities in the dump should not fail those asserts.

## Docs

Replace the single compatibility row (do not add a second historical row) in:

- [`README.md`](README.md)
- [`crates/robocomp_rapier3d/README.md`](crates/robocomp_rapier3d/README.md)
- [`crates/robocomp_avian3d/README.md`](crates/robocomp_avian3d/README.md)

New row: Bevy **0.19**, rapier **0.36**, avian **0.7**, robocomp crates still **0.1**.

## Verification

- `cargo test --workspace` (matches the root README; also covers `--all-features` if that is what [`JUSTFILE`](JUSTFILE) runs).
- `cargo check -p robocomp_rapier3d --examples` and `cargo check -p robocomp_avian3d --examples`, including the `bsn!` rewrites.
- Graphical examples are not run headlessly. If a compile error is example-only, fix it from the compiler output.