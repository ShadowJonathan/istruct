pub mod v1 {
    use std::sync::Arc;

    use async_trait::async_trait;
    use axum::{
        extract::{Extension, Path},
        routing::{delete, post},
        AddExtensionLayer, Json,
    };

    use crate::{api::ApiBase, id::DeviceId, router::VersionedRouter};

    #[async_trait]
    pub trait NetworkDevApi: ApiBase {
        async fn create_nat(&self) -> DeviceId;

        async fn delete_nat(&self, device: DeviceId) -> anyhow::Result<()>;
    }

    pub fn convert<A: NetworkDevApi>(api: A) -> VersionedRouter {
        let router = axum::Router::new()
            .route(
                "/nat",
                post(|Extension::<Arc<A>>(api)| async move { Json(api.create_nat().await) }),
            )
            .route(
                "/nat/:did",
                delete(|Extension::<Arc<A>>(api), Path(did)| async move {
                    api.delete_nat(did).await.map_err(|e| {
                        (
                            axum::http::StatusCode::CONFLICT,
                            format!("failed to delete NAT device: {}", e),
                        )
                    })
                }),
            )
            .layer(AddExtensionLayer::new(Arc::new(api)));

        VersionedRouter::new(router, "is.network.device", 0, 1)
    }
}
