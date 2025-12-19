use anyhow::Result;
use clap::Parser;
use clap::Subcommand;
#[cfg(feature = "web_admin_panel")]
use esbuild_metafile::instance::initialize_instance;
use log::info;
use paddler::cmd::agent::Agent;
use paddler::cmd::balancer::Balancer;
use paddler::cmd::handler::Handler as _;
use tokio::signal::unix::SignalKind;
use tokio::signal::unix::signal;
use tokio::sync::oneshot;

#[cfg(feature = "web_admin_panel")]
pub const ESBUILD_META_CONTENTS: &str = include_str!("../esbuild-meta.json");

pub const CUDA_DISCLAIMER_DOCS: &str = " \
    This software includes NVIDIA CUDA runtime components, subject to the NVIDIA CUDA Toolkit End User License Agreement: \
    https://docs.nvidia.com/cuda/eula/index.html\n\
    This software contains source code provided by NVIDIA Corporation.\n\
    Paddler is not affiliated with, endorsed by, or sponsored by NVIDIA Corporation.";

#[derive(Parser)]
#[command(arg_required_else_help(true), version, about, long_about = None)]
/// LLMOps platform for hosting and scaling open-source LLMs in your own infrastructure
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

// CMAKE_CUDA_ARCHITECTURES=native CMAKE_CUDA_COMPILER_FORCED=ON CMAKE_CUDA_COMPILER=/run/current-system/sw/bin/nvcc CC=/nix/store/p20mdvw9izjilv86xlj32cghafhpnrks-gcc-wrapper-12.4.0/bin/gcc CCX=/nix/store/p20mdvw9izjilv86xlj32cghafhpnrks-gcc-wrapper-12.4.0/bin/gcc
//   CMAKE_GENERATOR="Unix Makefiles" CUDACXX=/run/current-system/sw/bin/nvcc cargo build --features cuda && nix-ld target/debug/paddler -h

#[expect(clippy::large_enum_variant)]
#[derive(Subcommand)]
enum Commands {
    /// Generates tokens and embeddings; connects to the balancer
    #[cfg_attr(
        feature = "cuda",
        doc = CUDA_DISCLAIMER_DOCS
    )]
    Agent(Agent),
    /// Distributes incoming requests among agents
    Balancer(Balancer),
}

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        let mut sigterm = signal(SignalKind::terminate()).expect("Failed to listen for SIGTERM");
        let mut sigint = signal(SignalKind::interrupt()).expect("Failed to listen for SIGINT");
        let mut sighup = signal(SignalKind::hangup()).expect("Failed to listen for SIGHUP");

        tokio::select! {
            _ = sigterm.recv() => info!("Received SIGTERM"),
            _ = sigint.recv() => info!("Received SIGINT (Ctrl+C)"),
            _ = sighup.recv() => info!("Received SIGHUP"),
        }

        shutdown_tx
            .send(())
            .expect("Failed to send shutdown signal");
    });

    match Cli::parse().command {
        Some(Commands::Agent(handler)) => Ok(handler.handle(shutdown_rx).await?),
        Some(Commands::Balancer(handler)) => {
            #[cfg(feature = "web_admin_panel")]
            initialize_instance(ESBUILD_META_CONTENTS);

            Ok(handler.handle(shutdown_rx).await?)
        }
        None => Ok(()),
    }
}
