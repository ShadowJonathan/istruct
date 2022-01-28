use istruct_common::{api, router::CompositeRouter};
use istruct_libvirt::client::{Client, ClientPuck};

fn main() -> anyhow::Result<()> {
    use tower_http::trace::{DefaultMakeSpan, TraceLayer};

    let puck = ClientPuck::create(|| {
        Client::new(
            "qemu:///system",
            "../istruct_data/db.persy",
            "../istruct_data/block/",
        )
        .unwrap()
    });

    let compute_api = api::compute::machine::v1::convert(puck.clone());
    let compute_dev_api = api::compute::machine::device::v1::convert(puck.clone());
    let devadm_api = api::compute::devadm::v1::convert(puck.clone());
    let storage_dev_api = api::storage::device::v1::convert(puck.clone());
    let network_dev_api = api::network::device::v1::convert(puck.clone());

    let composite = CompositeRouter::new_with([
        compute_api,
        compute_dev_api,
        devadm_api,
        storage_dev_api,
        network_dev_api,
    ])?;

    tracing_subscriber::fmt::init();

    let router = dbg!(composite.assemble())
        .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default()));

    let rt = tokio::runtime::Runtime::new()?;

    let sock = "[::]:8989".parse().unwrap();

    rt.block_on(async {
        axum::Server::bind(&sock)
            .serve(router.into_make_service())
            .await
    })?;

    Ok(())
}
