pub mod v1 {
    use std::sync::Arc;

    use crate::{api::ApiBase, id::DeviceId, router::VersionedRouter};
    use async_trait::async_trait;
    use axum::{
        extract::{Extension, Path},
        http::StatusCode,
        routing::get,
        AddExtensionLayer, Json,
    };
    use serde::{Deserialize, Serialize};

    // note: VCPU and Memory devices have same lifetime as machine

    #[async_trait]
    pub trait MachineDevApi: ApiBase {
        async fn get_memory(&self, device: DeviceId) -> Option<MemoryDevice>;

        async fn set_memory(&self, device: DeviceId, memory: MemoryDevice) -> anyhow::Result<()>;

        async fn get_cpu(&self, device: DeviceId) -> Option<CpuDevice>;

        async fn set_cpu(&self, device: DeviceId, cpu: CpuDevice) -> anyhow::Result<()>;
    }

    pub fn convert<A: MachineDevApi>(api: A) -> VersionedRouter {
        let router = axum::Router::new()
            .route(
                "/mem/:did",
                get(|Extension::<Arc<A>>(api), Path(did)| async move {
                    api.get_memory(did)
                        .await
                        .map(Json)
                        .ok_or(StatusCode::NOT_FOUND)
                })
                .patch(
                    |Extension::<Arc<A>>(api), Path(did), Json(device)| async move {
                        api.set_memory(did, device).await.map_err(|e| {
                            (
                                axum::http::StatusCode::CONFLICT,
                                format!("failed to set memory: {}", e),
                            )
                        })
                    },
                ),
            )
            .route(
                "/cpu/:did",
                get(|Extension::<Arc<A>>(api), Path(did)| async move {
                    api.get_cpu(did)
                        .await
                        .map(Json)
                        .ok_or(StatusCode::NOT_FOUND)
                })
                .patch(
                    |Extension::<Arc<A>>(api), Path(did), Json(device)| async move {
                        api.set_cpu(did, device).await.map_err(|e| {
                            (
                                axum::http::StatusCode::CONFLICT,
                                format!("failed to set cpu: {}", e),
                            )
                        })
                    },
                ),
            )
            .layer(AddExtensionLayer::new(Arc::new(api)));

        return VersionedRouter::new(router, "is.compute.machine.device", 0, 1);
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub struct CpuDevice {
        pub cores: u64,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct MemoryDevice {
        pub bytes: u64,
    }
}
