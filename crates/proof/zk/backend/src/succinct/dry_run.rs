//! Dry-run [`ZkProver`] that completes instantly with an empty proof.

use async_trait::async_trait;
use base_proof_zk_host::{
    ZkProofRequestKind, ZkProver, ZkProverError, ZkSessionRecorder, ZkSessionState,
};
use base_prover_service_protocol::{
    BackendSessionState, ProofResult, SessionType, SnarkGroth16ProofResult, ZkProofResult, ZkVm,
};

/// [`ZkProver`] that completes instantly with an empty proof payload.
#[derive(Debug, Clone, Copy, Default)]
pub struct DryRunZkProver;

impl DryRunZkProver {
    /// Derive the deterministic backend session id for a request.
    pub fn backend_session_id(request: &ZkProofRequestKind, request_session_id: &str) -> String {
        Self::backend_session_id_for_type(request.initial_session_type(), request_session_id)
    }

    /// Derive the deterministic backend session id for a stage.
    pub fn backend_session_id_for_type(
        session_type: SessionType,
        request_session_id: &str,
    ) -> String {
        let session_type = match session_type {
            SessionType::Stark => "stark",
            SessionType::Snark => "snark",
        };
        format!("dry-run-{session_type}-{request_session_id}")
    }
}

#[async_trait]
impl ZkProver for DryRunZkProver {
    async fn submit(
        &self,
        request: &ZkProofRequestKind,
        request_session_id: &str,
    ) -> Result<String, ZkProverError> {
        Ok(Self::backend_session_id(request, request_session_id))
    }

    async fn poll(
        &self,
        _session_type: SessionType,
        _backend_session_id: &str,
    ) -> Result<ZkSessionState, ZkProverError> {
        Ok(ZkSessionState::Completed)
    }

    async fn submit_next(
        &self,
        request: &ZkProofRequestKind,
        session_recorder: &(dyn ZkSessionRecorder + Send + Sync),
        completed_session_type: SessionType,
        request_session_id: &str,
        _completed_backend_session_id: &str,
    ) -> Result<Option<(SessionType, String)>, ZkProverError> {
        if !request.is_snark_groth16() || completed_session_type != SessionType::Stark {
            return Ok(None);
        }

        let backend_session_id =
            Self::backend_session_id_for_type(SessionType::Snark, request_session_id);
        session_recorder
            .record_backend_session(
                SessionType::Snark,
                backend_session_id.clone(),
                BackendSessionState::Running,
            )
            .await?;

        Ok(Some((SessionType::Snark, backend_session_id)))
    }

    async fn download(
        &self,
        session_type: SessionType,
        _backend_session_id: &str,
    ) -> Result<ProofResult, ZkProverError> {
        let zk_proof = ZkProofResult { zk_vm: ZkVm::Sp1, proof: Vec::new().into() };

        if session_type == SessionType::Snark {
            Ok(ProofResult::SnarkGroth16(SnarkGroth16ProofResult { proof: zk_proof }))
        } else {
            Ok(ProofResult::Compressed(zk_proof))
        }
    }
}

#[cfg(test)]
mod tests {
    use base_prover_service_protocol::{SnarkGroth16ProofRequest, ZkProofRequest, ZkVm};

    use super::*;

    struct TestSessionRecorder;

    #[async_trait::async_trait]
    impl ZkSessionRecorder for TestSessionRecorder {
        async fn record_backend_session(
            &self,
            _session_type: SessionType,
            _backend_session_id: String,
            _state: BackendSessionState,
        ) -> Result<(), ZkProverError> {
            Ok(())
        }
    }

    fn zk_request() -> ZkProofRequest {
        ZkProofRequest {
            start_block_number: 1,
            number_of_blocks_to_prove: 1,
            sequence_window: None,
            l1_head: None,
            intermediate_root_interval: None,
            zk_vm: ZkVm::Sp1,
        }
    }

    fn compressed() -> ZkProofRequestKind {
        ZkProofRequestKind::Compressed(zk_request())
    }

    fn snark_groth16() -> ZkProofRequestKind {
        ZkProofRequestKind::SnarkGroth16(SnarkGroth16ProofRequest {
            proof: zk_request(),
            prover_address: Default::default(),
        })
    }

    #[tokio::test]
    async fn dry_run_completes_with_empty_proof() {
        let prover = DryRunZkProver;
        let id = prover.submit(&compressed(), "session-1").await.unwrap();

        assert_eq!(id, "dry-run-stark-session-1");
        assert_eq!(prover.poll(SessionType::Stark, &id).await.unwrap(), ZkSessionState::Completed);

        match prover.download(SessionType::Stark, &id).await.unwrap() {
            ProofResult::Compressed(proof) => assert!(proof.proof.is_empty()),
            other => panic!("expected compressed proof, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn dry_run_submits_next_snark_groth16_to_empty_snark_proof() {
        let prover = DryRunZkProver;
        let request = snark_groth16();
        let id = prover.submit(&request, "session-1").await.unwrap();

        assert_eq!(id, "dry-run-stark-session-1");
        assert_eq!(
            prover
                .submit_next(&request, &TestSessionRecorder, SessionType::Stark, "session-1", &id)
                .await
                .unwrap(),
            Some((SessionType::Snark, "dry-run-snark-session-1".to_owned()))
        );

        let snark_id = DryRunZkProver::backend_session_id_for_type(SessionType::Snark, "session-1");
        match prover.download(SessionType::Snark, &snark_id).await.unwrap() {
            ProofResult::SnarkGroth16(proof) => assert!(proof.proof.proof.is_empty()),
            other => panic!("expected SNARK Groth16 proof, got {other:?}"),
        }
    }
}
