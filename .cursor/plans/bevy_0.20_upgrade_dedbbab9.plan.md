---
name: Bevy 0.20 upgrade
overview: Upgrade the workspace from Bevy 0.19 to 0.20 (resolves to 0.20.1), bump the robocomp crates to 0.3.0, and move Rapier, Avian, Skein, and the inspector to the matching 0.20 releases, then update BSN syntax and observer triggers and fix whatever still fails to compile.
todos:
  - id: bump-deps
    content: Pin workspace Bevy 0.20 and matching Rapier 0.37, Avian 0.8, Skein 0.7, and inspector 0.38. Bump the workspace package version to 0.3.0. Do not set rust-version or a toolchain file.
    status: completed
  - id: bsn-syntax
    content: Update both backends' code-built examples and scene_setup to @ scene refs, drop template_value, and separate entities with --. Remove VariantDefaults from RcJointKind and RcJointMotorControlConfig.
    status: completed
  - id: observers
    content: Change On<Add, Collider> to On<Add<Collider>> in the Rapier and Avian backend tests.
    status: completed
  - id: fix-compile
    content: cargo check --workspace --all-targets and fix remaining 0.20 breaks (commands, features, physics APIs).
    status: completed
  - id: update-docs
    content: Prepend a Bevy 0.20 / robocomp 0.3 compatibility row in the three READMEs. Keep the 0.19 and 0.18 rows. Update quick-start dependency versions from 0.2 to 0.3.
    status: completed
  - id: verify
    content: Run workspace tests and example checks for both backends.
    status: completed
isProject: false
---

# Upgrade the workspace to Bevy 0.20

Stay on crates.io. Do not set `rust-version` and do not add a `rust-toolchain` file. Bevy 0.20’s crate floor is Rust 1.97.1; whatever stable is installed is enough, same policy as the [0.19 upgrade](.cursor/plans/bevy_0.19_upgrade_50bca553.plan.md). `Cargo.lock` stays gitignored.

Bump the workspace package version from `0.2.0` to `0.3.0` in [`Cargo.toml`](Cargo.toml): both `[workspace.package] version` and the `robocomp` path entry in `[workspace.dependencies]`. Member crates already use `version.workspace = true`, so `robocomp`, `robocomp_rapier3d`, and `robocomp_avian3d` all become 0.3.0. This is a breaking change because the crates now require Bevy 0.20. Do not publish in this upgrade.

## Dependency pins

All versions live in `[workspace.dependencies]` in [`Cargo.toml`](Cargo.toml). Member crates already use `workspace = true`.

- `bevy`: `0.19` → `0.20` (resolves to 0.20.1). Keep `default-features = false`. Same caret style as today.
- `bevy_rapier3d`: `0.36` → `0.37` (Bevy `^0.20`, rapier `^0.36.1`).
- `avian3d`: `0.7` → `0.8` (Bevy `^0.20`). [0.7 → 0.8 guide](https://github.com/avianphysics/avian/blob/v0.8.0/migration-guides/0.7-to-0.8.md).
- `bevy_skein`: `0.6` → `0.7` (published 2026-10-26). Its Bevy crates are `^0.20.0`, which is `>=0.20.0, <0.21.0`, so it resolves with Bevy 0.20.1. `0.7.0-rc.1` was the pre-release that depended on `^0.20.0-rc.1` and rejected 0.20.1; do not use it. `insert_skein_preset` in [`src/presets_plugin.rs`](src/presets_plugin.rs) keeps the same signature.
- `bevy-inspector-egui`: `0.37` → `0.38` (Bevy 0.20, `bevy_egui` 0.43).
- `petgraph` stays at `0.8`.

## BSN syntax

[0.19 → 0.20 guide](https://bevy.org/learn/migration-guides/0-19-to-0-20/): scene references need `@`, `template_value` goes away, and entity lists use `--` instead of commas and parentheses. `()` and `,` are deprecated. Apply this in both backends (the Rapier and Avian copies match):

- [`crates/robocomp_rapier3d/examples/helpers/scene_setup.rs`](crates/robocomp_rapier3d/examples/helpers/scene_setup.rs) and the Avian copy: `bsn_list! { ... -- ... }`, and `template_value(camera_transform)` / `template_value(light_transform)` become the `Transform` values themselves.
- `simple_rigid_bodies_*`, `revolute_*`, `prismatic_ctrl_*`, `multi_joints_ctrl_*`: `({robot})` → `@{robot}`; scene helpers such as `link(...)` and `joint(...)` become `@link(...)` / `@joint(...)`; sibling entities inside `Children [ ]` are separated with `--`.

Keep `{...}` inside component constructors (`Name({format!(...)})`, `RcLink { name: {RdName::new(name)}, ... }`, `Mesh3d({mesh_hdl})`). Those are component values, not scene slots. Examples already fill every enum field (`parent`, `child`, `axis`), so the removal of partial variant defaults does not change those patches.

Drop `VariantDefaults` from [`src/rc/mod.rs`](src/rc/mod.rs): the import, and the derives on `RcJointKind` and `RcJointMotorControlConfig`. Both already implement `Default + Clone`, which is what 0.20 BSN requires. Update the `RcJointKind` doc that says the default exists so the enum can be written in `bsn!`.

Leave [`crates/robocomp_test_utils/src/scenarios.rs`](crates/robocomp_test_utils/src/scenarios.rs) on `children!`. Those are tests, not examples.

Leave the skein examples on `WorldAssetRoot` + `GltfAssetLabel::Scene(0)` in [`revolute_skein_rapier3d.rs`](crates/robocomp_rapier3d/examples/revolute_skein_rapier3d.rs) and [`revolute_skein_avian3d.rs`](crates/robocomp_avian3d/examples/revolute_skein_avian3d.rs). The 0.20 guide does not move glTF loading onto `bsn!`. [`src/rc/mod.rs`](src/rc/mod.rs) already documents `RcSceneRoot` against `WorldAssetRoot`.

## Observers

`On<Add, Collider>` → `On<Add<Collider>>` in [`crates/robocomp_rapier3d/tests/rapier_backend.rs`](crates/robocomp_rapier3d/tests/rapier_backend.rs) and [`crates/robocomp_avian3d/tests/avian_backend.rs`](crates/robocomp_avian3d/tests/avian_backend.rs).

## Compile-driven fixes

After the pins and the edits above, `cargo check --workspace --all-targets` and fix whatever the compiler reports. Expected non-changes, touched only on error:

- Feature set stays `bevy_render` + `bevy_log` on the libraries, and dev `2d` / `3d` / `ui` / `bevy_dev_tools`. `bevy_shape` moved out of `bevy_math`, but `Cuboid` / `Plane3d` remain in the prelude. Do not turn default features back on.
- [`src/cleanup_manager.rs`](src/cleanup_manager.rs): `queue_handled`, `despawn`, and `despawn_related::<Children>` are not in the 0.20 guide.
- [`crates/robocomp_test_utils/src/harness.rs`](crates/robocomp_test_utils/src/harness.rs): `WorldSerializationPlugin`, `DynamicWorld::from_world`, and `bevy::mesh::MeshPlugin` are unchanged in the guide.
- Rapier: current 0.37 docs still have `PhysicsSet::SyncBackend`, `WriteRapierContext::single_mut`, and the `from_bevy_mesh` / `Ccd::enabled()` calls in [`crates/robocomp_rapier3d/src/controller_plugin.rs`](crates/robocomp_rapier3d/src/controller_plugin.rs) and [`pre_processor_plugin.rs`](crates/robocomp_rapier3d/src/pre_processor_plugin.rs). The published changelog stops at 0.34; treat 0.37 as a Bevy bump unless a symbol this code calls was renamed.
- Avian: this crate does not enable `f64`, so the `Scalar` → `Real` rename does not apply. `PhysicsSystems::Prepare` is not renamed. `RcCcd` is still ignored (`_rc_ccd` in [`crates/robocomp_avian3d/src/pre_processor_plugin.rs`](crates/robocomp_avian3d/src/pre_processor_plugin.rs)); do not wire `SweptCcd` in this upgrade. Collider-context, solver-body, and joint-damping changes are unused. `convex_hull_from_mesh` / `trimesh_from_mesh` / `convex_decomposition_from_mesh`, `ColliderDensity`, and `SleepingDisabled` change only if they no longer compile (`Collider::convex_hull` and `convex_decomposition` now borrow their slices).
- Example helpers: `FpsOverlayPlugin`, `GlobalAmbientLight`, `EguiPlugin::default()`, `WorldInspectorPlugin`. No documented break; `bevy_egui` did move to 0.43 with inspector 0.38, so fix `scene_setup.rs` if `EguiPlugin::default()` fails.

No shaders, pointer events, UI text, or `iter_many` call sites in this repo.

## Docs

Prepend a row and keep the existing 0.19 and 0.18 rows in:

- [`README.md`](README.md)
- [`crates/robocomp_rapier3d/README.md`](crates/robocomp_rapier3d/README.md)
- [`crates/robocomp_avian3d/README.md`](crates/robocomp_avian3d/README.md)

New top row: Bevy **0.20**, rapier **0.37**, avian **0.8**, robocomp crates **0.3**.

Also change the quick-start dependency snippets in those three READMEs from `"0.2"` to `"0.3"` (`robocomp_rapier3d` and `robocomp_avian3d`).

## Verification

- `cargo test --workspace --all-features` (what [`JUSTFILE`](JUSTFILE) `test-all` runs).
- `cargo check -p robocomp_rapier3d --examples` and `cargo check -p robocomp_avian3d --examples`.
- Graphical examples are not run headlessly. If a compile error is example-only, fix it from the compiler output.
