# Requirement: Migrate ExEx gRPC Extension to base v1.2.0

## Background

The `dev-exex-internal-txs-v2.3.0` branch (based on base `v1.1.1`) implements gRPC ExEx extension functionality:
- gRPC ExEx crate and node startup integration
- Per-transaction call trace collection during block execution

base upstream released `v1.2.0`. The above functionality needs to be migrated to the new version.

## Objective

Migrate all ExEx-related functionality from branch `dev-exex-internal-txs-v2.3.0` to the new branch `dev-exex-support-for-base-v1.2.0` (based on base `v1.2.0`), with no behavioral changes.

## Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| F1 | System shall include a `grpc-exex` crate providing gRPC ExEx extension capability | Must |
| F2 | System shall integrate gRPC ExEx extension into the node startup pipeline (`standard_node.rs`) | Must |
| F3 | System shall collect per-transaction call traces during block execution (`engine-tree/validator.rs`) | Must |
| F4 | System shall depend on `bnmlsp/reth` branch `dev-exex-internal-txs-v2.3.0` as reth override | Must |
| F5 | System shall depend on `bnmlsp/reth-remote-exex` tag `for_reth_v2.3.0_base_support` | Must |

## Non-Functional Requirements

| Item | Conclusion |
|------|------------|
| Performance | Not applicable (flat migration, no behavioral change) |
| Availability | Not applicable |
| Security | Not applicable |
| Scalability | Not applicable |
| Observability | Not applicable |
| Data retention | Not applicable |
| Compatibility | Must be compatible with base v1.2.0 API and startup pipeline changes |

## Acceptance Criteria

| ID | Condition |
|----|-----------|
| AC1 | `cargo check -p base-grpc-exex` passes |
| AC2 | `cargo check -p base-execution-cli` passes (validates startup integration) |
| AC3 | `cargo check -p base-engine-tree` passes (validates call trace integration) |
| AC4 | `Cargo.toml` reth dependencies point to `bnmlsp/reth` branch `dev-exex-internal-txs-v2.3.0` |
| AC5 | `Cargo.toml` `reth-remote-exex` dependency points to tag `for_reth_v2.3.0_base_support` |
| AC6 | Full workspace `cargo check` passes |

## Out of Scope

- Deletion or archival of the old branch `dev-exex-internal-txs-v2.3.0`
- Version upgrade of the `bnmlsp/reth` repository
- Any feature additions or behavioral changes
- Adding new unit tests (original commits had no tests; flat migration does not add any)

## Constraints and Assumptions

- base v1.2.0 and v1.1.1 depend on the same reth version (v2.3.0); `bnmlsp/reth` branch requires no changes
- `standard_node.rs` requires adaptation to v1.2.0 structural changes (file was refactored)

## Change History

- [2026-08-21] Initial version created
