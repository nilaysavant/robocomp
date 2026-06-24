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

# List all files to be packaged for all workspace crates
package-list-all:
    cargo package --list --workspace

# Run all pre-publish checks
pre-release-checks: package-list-all test-all build-all docs-all

# Release all workspace crates
release-all:
    cargo release --workspace --all-features

# Clean all build artifacts
clean:
    cargo clean
