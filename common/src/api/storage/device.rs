pub mod v1 {
    use std::sync::Arc;

    use async_trait::async_trait;
    use axum::{
        extract::{Extension, Path},
        http::StatusCode,
        routing::{get, post},
        AddExtensionLayer, Json,
    };
    use serde::{Deserialize, Serialize};

    use crate::{api::ApiBase, id::DeviceId, router::VersionedRouter};

    #[async_trait]
    pub trait StorageDevApi: ApiBase {
        async fn get_block(&self, device: DeviceId) -> Option<BlockDevice>;

        async fn create_block(&self, block: BlockDevice) -> DeviceId;

        async fn delete_block(&self, device: DeviceId) -> anyhow::Result<()>;
    }

    pub fn convert<A: StorageDevApi>(api: A) -> VersionedRouter {
        let router = axum::Router::new()
            .route(
                "/block",
                post(|Extension::<Arc<A>>(api), Json(block)| async move {
                    Json(api.create_block(block).await)
                }),
            )
            .route(
                "/block/:did",
                get(|Extension::<Arc<A>>(api), Path(did)| async move {
                    api.get_block(did)
                        .await
                        .map(Json)
                        .ok_or(StatusCode::NOT_FOUND)
                })
                .delete(|Extension::<Arc<A>>(api), Path(did)| async move {
                    api.delete_block(did).await.map_err(|e| {
                        (
                            StatusCode::CONFLICT,
                            format!("failed to delete block device: {}", e),
                        )
                    })
                }),
            )
            .layer(AddExtensionLayer::new(Arc::new(api)));

        VersionedRouter::new(router, "is.storage.device", 0, 1)
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct BlockDevice {
        pub bytes: u64,
    }
}
