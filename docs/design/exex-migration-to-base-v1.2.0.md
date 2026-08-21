# Design: Migrate ExEx gRPC Extension to base v1.2.0

## Background

Migrate ExEx functionality from `dev-exex-internal-txs-v2.3.0` (based on base `v1.1.1`) to `dev-exex-support-for-base-v1.2.0` (based on `v1.2.0`).

## Objective

Restore full gRPC ExEx functionality on the base v1.2.0 codebase. Compilation must pass; behavior remains unchanged.

## Module Breakdown

| Module | Change Type | Notes |
|--------|-------------|-------|
| `crates/execution/grpc-exex/` | New (entire crate) | Direct copy, no adaptation needed |
| `crates/execution/cli/` | Modify | Add `install_ext::<GrpcExExExtension>(())` in v1.2.0's `standard_node.rs` |
| `crates/execution/engine-tree/` | Modify | `validator.rs` needs call trace logic added (no upstream conflict — file unchanged between v1.1.1→v1.2.0); Cargo.toml needs new deps |

## Dependency Changes

Workspace `Cargo.toml` additions/modifications:

1. **Add** `alloy-rpc-types-trace = { version = "2.0.5", default-features = false }` — workspace declaration
2. **Add** `reth-remote-exex = { git = "https://github.com/bnmlsp/reth-remote-exex.git", tag = "for_reth_v2.3.0_base_support", default-features = false, features = ["base"] }` — workspace declaration
3. **Add** `base-grpc-exex = { path = "crates/execution/grpc-exex" }` — workspace declaration
4. **Modify** all `reth-*` dependencies from `paradigmxyz/reth tag=v2.3.0` to `bnmlsp/reth branch=dev-exex-internal-txs-v2.3.0`
5. **Add** `crates/execution/grpc-exex` to workspace members

`engine-tree/Cargo.toml` additions:
- `revm-inspectors.workspace = true`
- `alloy-rpc-types-trace.workspace = true`
- `reth-chain-state` changed to `{ workspace = true, features = ["traces"] }`

`execution/cli/Cargo.toml` addition:
- `base-grpc-exex.workspace = true`

## Interface Definition

No new interfaces. `GrpcExExExtension` implements the existing `BaseNodeExtension` trait and registers via `runner.install_ext::<GrpcExExExtension>(())`, consistent with other extensions in v1.2.0.

## Implementation Steps

All code is sourced from this repo's `dev-exex-internal-txs-v2.3.0` branch.

1. Copy `crates/execution/grpc-exex/` directory (Cargo.toml + README.md + src/lib.rs) from `dev-exex-internal-txs-v2.3.0`
2. Modify workspace `Cargo.toml`: add workspace member, add dependency declarations, replace reth source
3. Modify `crates/execution/engine-tree/Cargo.toml`: add trace-related dependencies
4. Modify `crates/execution/engine-tree/src/validator.rs`: add call trace collection logic (from `dev-exex-internal-txs-v2.3.0`)
5. Modify `crates/execution/cli/Cargo.toml`: add `base-grpc-exex` dependency
6. Modify `crates/execution/cli/src/standard_node.rs`: add import and `install_ext` call
7. Regenerate `Cargo.lock`

## Alternative Approaches

| Approach | Pros/Cons |
|----------|-----------|
| **Manual file-by-file migration** (chosen) | Controllable, avoids meaningless conflict resolution; suitable when Cargo.toml diffs are large |
| Cherry-pick + resolve conflicts | Cargo.toml/Cargo.lock conflicts are dense and error-prone; manual is cleaner |

Chosen: manual file-by-file migration. Cherry-pick would produce heavy conflicts in Cargo.toml (238-line diff) and standard_node.rs (structural refactor).

## Test Plan

| ID | Scenario | Input | Expected Result |
|----|----------|-------|-----------------|
| T1 | grpc-exex crate standalone compilation | `cargo check -p base-grpc-exex` | exit 0, no errors |
| T2 | execution-cli compilation with ExEx integration | `cargo check -p base-execution-cli` | exit 0, no errors |
| T3 | engine-tree compilation with call trace logic | `cargo check -p base-engine-tree` | exit 0, no errors |
| T4 | Full workspace compilation | `cargo check` | exit 0 |
| T5 | reth dependency source verification | Inspect Cargo.toml reth-* git source | All point to `bnmlsp/reth` branch `dev-exex-internal-txs-v2.3.0` |
| T6 | reth-remote-exex dependency verification | Inspect Cargo.toml | Points to tag `for_reth_v2.3.0_base_support` |

## Known Risks

- `validator.rs` is unchanged between v1.1.1 and v1.2.0, but v1.2.0's `engine-tree/Cargo.toml` lacks the `traces` feature and `revm-inspectors` — must be added manually
- `standard_node.rs` was significantly refactored in v1.2.0; the `install_ext` call must be inserted at the correct location

## Out of Scope

Same as requirement document.
