# Requirement: Call Trace Collection in Base Block Execution

## Background

The `reth-remote-exex` gRPC service fully implements call trace serialization and push logic. On ETH nodes this works correctly. However, after deploying the Base node, call traces are always empty. The root cause is that Base's block validator (`base-engine-tree/src/validator.rs`) uses `NoOpInspector` during block execution and does not collect trace data.

## Objective

Enable the Base node to collect per-transaction call traces during block execution and attach them to `ExecutedBlock`, so that downstream `canonical_state_stream()` pushes `Chain` objects containing call trace data.

## Functional Requirements

| ID | Description | Priority |
|----|-------------|----------|
| FR-1 | Base node block execution must use `TracingInspector` instead of `NoOpInspector` to collect per-transaction call traces (CallFrame) | Must |
| FR-2 | Collected call traces must be assigned to `ExecutedBlock.call_traces` field | Must |
| FR-3 | The original `execute_transactions` method must be retained (not deleted) for upstream merge compatibility | Should |

## Non-Functional Requirements

| Category | Decision |
|----------|----------|
| Performance | Not quantified this iteration; accept `TracingInspector` overhead (runtime toggle can be added later) |
| Compatibility | No public interface changes; must not affect existing Base node behavior (block production, consensus, RPC) |
| Observability | Not in scope |
| Security | Not in scope |

## Acceptance Criteria

| ID | Criterion |
|----|-----------|
| AC-1 | `cargo check -p base-engine-tree` compiles successfully |
| AC-2 | `cargo check -p base-reth-node` compiles successfully (confirms feature unification is correct) |
| AC-3 | After deployment, go-consumer with `--call-traces` flag receives non-zero call traces for blocks containing transactions (`call_traces: N blocks` where N > 0) |

## Out of Scope

- Runtime toggle (enable/disable tracing on demand)
- Performance benchmarking
- Call trace data correctness verification (existing `go-trace-verifier` tool handles this independently post-deployment)

## Dependencies

- `bnmlsp/reth` branch `dev-exex-internal-txs-v2.3.0`: `reth-chain-state/traces` feature
- `revm-inspectors` crate (already in workspace)
- `alloy-rpc-types-trace` crate (already in workspace)

## Assumptions

- `CachedExecutor` wrapper does not constrain inspector type; `evm_mut()` passes through to inner executor
- Base's `execute_transaction` returns `GasOutput` (consistent with reth)

## Change History

- [2026-06-27] Initial requirement created / Base call traces empty after deployment / Scope: base-engine-tree crate
