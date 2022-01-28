pub mod device;

pub mod v1 {
    use std::{collections::HashMap, sync::Arc};

    use async_trait::async_trait;
    use axum::{
        extract::{Extension, Path},
        http::StatusCode,
        routing::{delete, get, post, put},
        AddExtensionLayer, Json,
    };
    use serde::{Deserialize, Serialize};

    use crate::{
        api::ApiBase,
        id::{DeviceId, MachineId},
        router::VersionedRouter,
    };

    #[async_trait]
    pub trait MachineApi: ApiBase {
        // todo check return types

        async fn act(&self, machine: MachineId, action: MachineAction);

        async fn status(&self, machine: MachineId) -> Option<MachineState>;

        // async fn get_attr(&self, machine: MachineId, attr: String) -> Option<Object>;

        // async fn set_attr(&self, machine: MachineId, attr: String, value: Object);

        // async fn delete_attr(&self, machine: MachineId, attr: String);

        // async fn patch_attrs(&self, machine: MachineId, attrs: HashMap<String, Object>);

        // async fn list_attrs(&self, machine: MachineId) -> Vec<String>;

        async fn dev_list(&self, machine: MachineId) -> Option<HashMap<DeviceId, String>>;

        async fn dev_attach(&self, machine: MachineId, device: DeviceId) -> anyhow::Result<()>;

        async fn dev_detach(&self, machine: MachineId, device: DeviceId) -> anyhow::Result<()>;

        async fn create(&self) -> MachineId;

        async fn destroy(&self, machine: MachineId) -> anyhow::Result<()>;

        async fn list(&self) -> Vec<MachineId>;

        // todo: temporary methods, change to devices, possibly storage ones
        // async fn get_cd(&self, machine: MachineId) -> Option<String>;

        // async fn set_cd(&self, machine: MachineId, cd: String) -> anyhow::Result<()>;

        // async fn rm_cd(&self, machine: MachineId) -> anyhow::Result<()>;
    }

    // todo: properly convert return values (fromrequest and all)
    // todo: make proc(?) macro for this, so manual mistakes arent made
    pub fn convert<A: MachineApi>(api: A) -> VersionedRouter {
        let router = axum::Router::new()
            .route(
                "/act/:mid/:action",
                post(|Extension::<Arc<A>>(api), Path((mid, action))| async move {
                    api.act(mid, action).await
                }),
            )
            .route(
                "/status/:mid",
                get(|Extension::<Arc<A>>(api), Path(mid)| async move {
                    api.status(mid).await.map(Json).ok_or(StatusCode::NOT_FOUND)
                }),
            )
            // todo: reimagine
            // .route(
            //     "/attr/:mid/io/:attr",
            //     get(|Extension::<Arc<A>>(api), Path((mid, attr))| async move {
            //         Json(api.get_attr(mid, attr).await)
            //     })
            //     .put(
            //         |Extension::<Arc<A>>(api), Path((mid, attr)), Json(value)| async move {
            //             api.set_attr(mid, attr, value).await;
            //         },
            //     )
            //     .delete(
            //         |Extension::<Arc<A>>(api), Path((mid, attr))| async move {
            //             api.delete_attr(mid, attr).await;
            //         },
            //     ),
            // )
            // .route(
            //     "/attr/:mid/io",
            //     patch(
            //         |Extension::<Arc<A>>(api), Path(mid), Json(attrs)| async move {
            //             api.patch_attrs(mid, attrs).await;
            //         },
            //     ),
            // )
            // .route(
            //     "/attr/:mid/ls",
            //     get(|Extension::<Arc<A>>(api), Path(mid)| async move {
            //         Json(api.list_attrs(mid).await)
            //     }),
            // )
            .route(
                "/dev/:mid",
                get(|Extension::<Arc<A>>(api), Path(mid)| async move {
                    api.dev_list(mid)
                        .await
                        .map(Json)
                        .ok_or(StatusCode::NOT_FOUND)
                }),
            )
            .route(
                "/dev/:mid/plug/:did",
                put(|Extension::<Arc<A>>(api), Path((mid, did))| async move {
                    api.dev_attach(mid, did).await.map_err(|e| {
                        (
                            axum::http::StatusCode::CONFLICT,
                            format!("failed to attach device: {}", e),
                        )
                    })
                })
                .delete(|Extension::<Arc<A>>(api), Path((mid, did))| async move {
                    api.dev_detach(mid, did).await.map_err(|e| {
                        (
                            axum::http::StatusCode::CONFLICT,
                            format!("failed to detach block device: {}", e),
                        )
                    })
                }),
            )
            .route(
                "/m",
                get(|Extension::<Arc<A>>(api)| async move { Json(api.list().await) })
                    .post(|Extension::<Arc<A>>(api)| async move { Json(api.create().await) }),
            )
            .route(
                "/m/:mid",
                delete(|Extension::<Arc<A>>(api), Path(mid)| async move {
                    api.destroy(mid).await.map_err(|e| {
                        (
                            axum::http::StatusCode::CONFLICT,
                            format!("failed to destroy device: {}", e),
                        )
                    })
                }),
            )
            // todo: refactor
            // .route(
            //     "/temp/:mid/cd",
            //     get(|Extension::<Arc<A>>(api), Path(mid)| async move {
            //         api.get_cd(mid).await.map(Json).ok_or(StatusCode::NOT_FOUND)
            //     })
            //     .post(|Extension::<Arc<A>>(api), Path(mid), Json(cd)| async move {
            //         api.set_cd(mid, cd).await.map_err(|e| {
            //             (
            //                 axum::http::StatusCode::CONFLICT,
            //                 format!("failed to set cdrom: {}", e),
            //             )
            //         })
            //     })
            //     .delete(|Extension::<Arc<A>>(api), Path(mid)| async move {
            //         api.rm_cd(mid).await.map_err(|e| {
            //             (
            //                 axum::http::StatusCode::CONFLICT,
            //                 format!("failed to set cdrom: {}", e),
            //             )
            //         })
            //     }),
            // )
            .layer(AddExtensionLayer::new(Arc::new(api)));

        VersionedRouter::new(router, "is.compute.machine", 0, 1)
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum MachineAction {
        ForceShutdown,
        ForceReset,
        Shutdown,
        Suspend,
        Resume,
        Boot,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "snake_case")]
    pub enum MachineState {
        Running,
        Suspended,
        Off,
        Error,
    }
}
