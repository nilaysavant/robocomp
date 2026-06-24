default: dev

# Run the multi_robots_rapier3d example by default
dev:
    cargo run -p robocomp_rapier3d --example multi_joints_ctrl_rapier3d

# Run all tests
test-all:
    cargo test --workspace --all-features

# Build all crates
build-all:
    cargo build --workspace

# Generate documentation for all workspace crates
docs-all:
    cargo doc --workspace --no-deps --all-features

# Run all pre-publish checks
pre-publish-checks: test-all build-all docs-all

# Clean all build artifacts
clean:
    cargo clean
