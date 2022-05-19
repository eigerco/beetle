use anyhow::Result;
use clap::Parser;
use iroh_gateway::{
    config::{Config, RpcConfig},
    core::Core,
    metrics,
};

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, required = false, default_value_t = 9050)]
    port: u16,
    #[clap(short, long)]
    writeable: bool,
    #[clap(short, long)]
    fetch: bool,
    #[clap(short, long)]
    cache: bool,
    #[clap(long = "no-metrics")]
    no_metrics: bool,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();

    // TODO: configurable
    let rpc_config = RpcConfig::default();
    let mut config = Config::new(
        args.writeable,
        args.fetch,
        args.cache,
        args.port,
        rpc_config,
    );
    config.set_default_headers();
    println!("{:#?}", config);

    iroh_metrics::init(metrics::metrics_config(args.no_metrics))
        .expect("failed to initialize metrics");
    metrics::register_counters();

    let handler = Core::new(config).await?;
    let core_task = tokio::spawn(async move {
        handler.serve().await;
    });

    iroh_util::block_until_sigint().await;
    core_task.abort();

    iroh_metrics::shutdown_tracing();
    Ok(())
}