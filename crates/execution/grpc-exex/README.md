# base-grpc-exex

gRPC ExEx extension that streams chain data to connected subscribers.

Implements `BaseNodeExtension` to mount the gRPC ExEx into the Base node startup pipeline. On node start, it spawns a gRPC server that streams `ExExNotification` data to all connected clients.
