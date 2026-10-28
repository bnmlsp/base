#![doc = include_str!("../README.md")]

use std::sync::Arc;

use base_node_runner::{BaseNodeExtension, FromExtensionConfig, NodeHooks};
use reth_exex::ExExNotification;
use reth_remote_exex::server::{
    remote_exex, start_grpc_server, BROADCAST_CHANNEL_CAPACITY,
};
use tokio::sync::broadcast;
use tracing::info;

type Primitives = base_common_consensus::BasePrimitives;

/// Extension that unconditionally mounts the gRPC ExEx into the Base node.
#[derive(Debug, Clone, Copy)]
pub struct GrpcExExExtension;

impl BaseNodeExtension for GrpcExExExtension {
    fn apply(self: Box<Self>, hooks: NodeHooks) -> NodeHooks {
        let notifications = Arc::new(
            broadcast::channel::<Arc<ExExNotification<Primitives>>>(BROADCAST_CHANNEL_CAPACITY).0,
        );
        let notif_exex = notifications.clone();

        hooks
            .install_exex("grpc-exex", move |ctx| async move {
                Ok(remote_exex(ctx, notif_exex))
            })
            .add_node_started_hook(move |node| {
                info!(addr = "0.0.0.0:10000", "Starting gRPC ExEx server");
                node.task_executor.spawn_critical_task("gRPC ExEx server", async move {
                    start_grpc_server(notifications)
                        .await
                        .unwrap_or_else(|e| panic!("gRPC ExEx server failed: {e}"))
                });
                Ok(())
            })
    }
}

impl FromExtensionConfig for GrpcExExExtension {
    type Config = ();

    fn from_config(_config: Self::Config) -> Self {
        Self
    }
}
