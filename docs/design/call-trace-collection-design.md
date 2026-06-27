# Design: Call Trace Collection in Base Block Execution

## Background

Base's block validator (`base-engine-tree/src/validator.rs`) was cloned from reth's `BasicEngineValidator` before traced execution was added to the bnmlsp/reth fork. It currently uses `NoOpInspector` and does not collect call traces. This design describes how to integrate `TracingInspector` to collect per-transaction call traces during block execution.

## Objective

Collect per-transaction `CallFrame` data during block execution and attach it to `ExecutedBlock`, enabling downstream `canonical_state_stream()` to include call traces in `Chain` notifications.

## Module Breakdown

Only one crate is modified: `base-engine-tree`.

| Module | Responsibility | Owner (read/write) |
|--------|---------------|-------------------|
| `validator.rs` | Block execution + trace collection | Write: engine tree; Read: chain-state |
| `cached_execution.rs` | Execution caching wrapper | No changes — generic over inspector |

## Interface Changes

### `execute_block` return type

```rust
// Before
fn execute_block(...) -> Result<(BlockExecutionOutput<BaseReceipt>, Vec<Address>, Receiver), InsertBlockErrorKind>

// After
fn execute_block(...) -> Result<(BlockExecutionOutput<BaseReceipt>, Vec<Address>, Receiver, Vec<CallFrame>), InsertBlockErrorKind>
```

### New method: `execute_transactions_traced`

```rust
fn execute_transactions_traced<E, Tx, InnerTx, Err>(
    &self,
    executor: E,
    transaction_count: usize,
    transactions: impl Iterator<Item = Result<Tx, Err>>,
    receipt_tx: &crossbeam_channel::Sender<IndexedReceipt<BaseReceipt>>,
    executed_tx_index: &AtomicUsize,
) -> Result<(E, Vec<Address>, Vec<CallFrame>), BlockExecutionError>
where
    E: BlockExecutor<Receipt = BaseReceipt>,
    E::Evm: alloy_evm::Evm<Inspector = TracingInspector>,
    Tx: alloy_evm::block::ExecutableTx<E> + alloy_evm::RecoveredTx<InnerTx>,
    InnerTx: TxHashRef,
    Err: core::error::Error + Send + Sync + 'static;
```

## Data Flow

```
execute_block
  ├── create EVM with TracingInspector (not NoOpInspector)
  ├── wrap in CachedExecutor (transparent to inspector type)
  ├── execute_transactions_traced
  │     ├── apply_pre_execution_changes
  │     ├── fuse inspector (discard system call traces)
  │     └── for each tx:
  │           ├── execute_transaction → GasOutput
  │           ├── inspector.geth_builder().geth_call_traces(gas_used) → CallFrame
  │           ├── inspector.fuse() (reset for next tx)
  │           └── stream receipt to background task
  └── return (output, senders, receipt_rx, call_traces)

validate_block_with_state (caller)
  ├── calls execute_block → receives call_traces
  ├── spawn_deferred_trie_task → ExecutedBlock
  └── executed_block.call_traces = Some(call_traces)  // behind #[cfg(feature = "traces")]
```

## Dependency Changes

### `crates/execution/engine-tree/Cargo.toml`

Add under `[dependencies]`:
```toml
revm-inspectors.workspace = true
alloy-rpc-types-trace.workspace = true
reth-chain-state = { workspace = true, features = ["traces"] }
```

Note: `revm-inspectors` and `alloy-rpc-types-trace` are already defined in the workspace root. Adding `features = ["traces"]` to `reth-chain-state` in this crate (not the workspace root) complies with base's CLAUDE.md rule: "Features must be enabled only by the individual crates or binaries that need them."

## CachedExecutor Interaction

`CachedExecutor` wraps `BaseBlockExecutor<E, ...>` and delegates `evm_mut()` to the inner executor. When a transaction hits the cache, it returns early **without** executing through the EVM. For cached transactions:

- No EVM execution occurs → no inspector state → no trace data
- The inspector's `fuse()` call resets it regardless, so state stays consistent

**Design decision**: For cached transactions, push an empty `CallFrame` (default). This is acceptable because:
1. Cached execution only occurs for flashblocks (pending transactions re-executed on the same parent)
2. The gRPC consumer receives traces for finalized blocks, where cached execution does not apply in practice

## Alternative Approaches

### Alternative 1: Feature-gated tracing (always-on vs opt-in)

**Chosen: Always-on** — `TracingInspector` is always used.

Rationale: Simplicity. The ETH version (bnmlsp/reth) uses always-on tracing. The overhead of `TracingInspectorConfig::none()` is minimal (only records call frames, not storage/memory). A runtime toggle adds complexity without proven need at this stage.

### Alternative 2: Modify reth's `BasicEngineValidator` to be used directly by Base

**Rejected**: Base's validator has significant customizations:
- `CachedExecutor` for flashblocks
- Custom precompile setup (Beryl precompiles)
- Base-specific types (`BaseEvm`, `BasePrimitives`, `BaseBlock`)
- Custom state root strategies and overlay logic

Replacing with reth's validator would require reth to know about all these Base-specific concerns.

## Known Limitations and Risks

1. **Performance overhead**: `TracingInspector` adds per-opcode call overhead. Measured at ~5-15% on ETH blocks. Base blocks are similar in complexity. Acceptable for current deployment; can add a toggle later if needed.

2. **Memory**: `Vec<CallFrame>` for a 400-tx block can be several MB. This lives in memory until the chain notification is consumed. Acceptable given server memory (16GB+).

3. **Cached transactions produce empty traces**: See CachedExecutor section above. Not a practical issue for finalized blocks.

4. **Upstream merge friction**: Adding `execute_transactions_traced` diverges from upstream. Mitigation: keep original `execute_transactions` unchanged, follow the same pattern as bnmlsp/reth (which keeps both methods side by side).

## Test Plan

| ID | Scenario | Input | Expected Output |
|----|----------|-------|-----------------|
| T-1 | Compilation check (engine-tree crate) | `cargo check -p base-engine-tree` | Compiles without errors |
| T-2 | Compilation check (full binary) | `cargo check -p base-reth-node` | Compiles without errors; feature unification propagates `traces` |
| T-3 | E2E: call traces present in gRPC stream | Deploy node, run `go-consumer --call-traces` on blocks with txs | `call_traces: 1 blocks` with non-zero frame count |
| T-4 | E2E: blocks without transactions | Empty blocks (no txs) | `call_traces: 1 blocks` with empty frames list (not 0 blocks) |
| T-5 | Error path: inspector fuse after system call | Pre-execution system calls (beacon root, L1 info deposit) | System call traces are discarded, only user tx traces collected |

Note: T-3 through T-5 are verified via deployment on the production node (44.255.247.71), not unit tests. The traced execution path is a direct port from the ETH version which has been validated in production.

## ADR: Introducing `revm-inspectors` dependency to `base-engine-tree`

- **Title**: Add `revm-inspectors` as a dependency for call trace collection
- **Status**: Adopted
- **Background**: `base-engine-tree` needs `TracingInspector` to collect per-transaction call traces. This type lives in `revm-inspectors`.
- **Decision**: Add `revm-inspectors` (already in workspace) as a direct dependency of `base-engine-tree`.
- **Rationale**: This is the same crate used by bnmlsp/reth's `BasicEngineValidator`. No alternative exists for this functionality. The crate is already in the workspace dependency graph (used by reth-engine-tree).
- **Consequences**: One additional compile-time dependency for `base-engine-tree`. No runtime cost beyond the tracing itself.
