pub mod v1 {
    use std::{sync::Arc, collections::HashMap};

    use async_trait::async_trait;
    use axum::{
        extract::{Extension, Path},
        routing::get,
        AddExtensionLayer, Json, http::StatusCode,
    };

    use crate::{api::ApiBase, id::DeviceId, router::VersionedRouter};

    pub type DeviceType = String;

    #[async_trait]
    pub trait DevAdmApi: ApiBase {
        async fn all(&self) -> HashMap<DeviceId, DeviceType>;

        async fn get_type(&self, dev: DeviceId) -> Option<DeviceType>;
    }

    pub fn convert<A: DevAdmApi>(api: A) -> VersionedRouter {
        let router = axum::Router::new()
            .route(
                "/all",
                get(|Extension::<Arc<A>>(api)| async move { Json(api.all().await) }),
            )
            .route(
                "/type/:did",
                get(|Extension::<Arc<A>>(api), Path(did)| async move {
                    api.get_type(did).await.map(Json).ok_or(StatusCode::NOT_FOUND)
                }),
            )
            .layer(AddExtensionLayer::new(Arc::new(api)));

        VersionedRouter::new(router, "is.compute.devadm", 0, 1)
    }
}
