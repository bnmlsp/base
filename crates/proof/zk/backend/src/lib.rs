#![doc = include_str!("../README.md")]

mod succinct;
pub use succinct::{
    ClusterSessionId, ClusterZkProver, ClusterZkProverConfig, DryRunZkProver, L1HeadSource,
    MOCK_PROOF_BYTES, MockZkProver, NetworkZkProver, NetworkZkProverConfig,
    OpSuccinctWitnessProvider, SuccinctClusterBackendConfig, SuccinctNetworkBackendConfig,
    SuccinctRpcConfig, SuccinctZkBackendConfig, SuccinctZkProverBuildError,
    SuccinctZkProverBuilder, WitnessError, WitnessParams,
};
