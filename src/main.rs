use clap::Parser;
use jsonrpsee::{core::RpcResult, proc_macros::rpc, tracing::level_filters::LevelFilter};
use reth::cli::{
    components::{RethNodeComponents, RethRpcComponents},
    config::RethRpcConfig,
    ext::{RethCliExt, RethNodeCommandConfig},
    Cli,
};
use reth_transaction_pool::TransactionPool;

use clap::{Subcommand, ValueEnum};
use era_test_node::logging_middleware::LoggingMiddleware;
use era_test_node::node::ShowCalls;
use era_test_node::node::{InMemoryNodeConfig, ShowGasDetails, ShowStorageLogs, ShowVMDetails};
use era_test_node::observability::LogLevel;
use era_test_node::observability::Observability;
use era_test_node::{
    fork::{ForkDetails, ForkSource},
    http_fork_source::HttpForkSource,
};

use era_test_node::node::InMemoryNode;

use std::fs::File;
use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
};

use futures::{
    channel::oneshot,
    future::{self},
    FutureExt,
};
use jsonrpc_core::MetaIoHandler;

use era_test_node::namespaces::{
    ConfigurationApiNamespaceT, DebugNamespaceT, EthNamespaceT, EthTestNodeNamespaceT,
    EvmNamespaceT, HardhatNamespaceT, NetNamespaceT, Web3NamespaceT, ZksNamespaceT,
};

#[allow(clippy::too_many_arguments)]
async fn build_json_http<
    S: std::marker::Sync + std::marker::Send + 'static + ForkSource + std::fmt::Debug + Clone,
>(
    addr: SocketAddr,
    log_level_filter: LevelFilter,
    node: InMemoryNode<S>,
) -> tokio::task::JoinHandle<()> {
    let (sender, recv) = oneshot::channel::<()>();

    let io_handler = {
        let mut io = MetaIoHandler::with_middleware(LoggingMiddleware::new(log_level_filter));

        io.extend_with(NetNamespaceT::to_delegate(node.clone()));
        io.extend_with(Web3NamespaceT::to_delegate(node.clone()));
        io.extend_with(ConfigurationApiNamespaceT::to_delegate(node.clone()));
        io.extend_with(DebugNamespaceT::to_delegate(node.clone()));
        io.extend_with(EthNamespaceT::to_delegate(node.clone()));
        io.extend_with(EthTestNodeNamespaceT::to_delegate(node.clone()));
        io.extend_with(EvmNamespaceT::to_delegate(node.clone()));
        io.extend_with(HardhatNamespaceT::to_delegate(node.clone()));
        io.extend_with(ZksNamespaceT::to_delegate(node));
        io
    };

    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(1)
            .build()
            .unwrap();

        let server = jsonrpc_http_server::ServerBuilder::new(io_handler)
            .threads(1)
            .event_loop_executor(runtime.handle().clone())
            .start_http(&addr)
            .unwrap();

        server.wait();
        let _ = sender;
    });

    tokio::spawn(recv.map(drop))
}

#[tokio::main]
async fn main() {
    let node: InMemoryNode<HttpForkSource> = InMemoryNode::new(
        None,
        None,
        InMemoryNodeConfig {
            show_calls: ShowCalls::None,
            show_storage_logs: ShowStorageLogs::None,
            show_vm_details: ShowVMDetails::None,
            show_gas_details: ShowGasDetails::None,
            resolve_hashes: false,
            system_contracts_options: era_test_node::system_contracts::Options::BuiltIn,
        },
    );

    let threads = build_json_http(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8011),
        LevelFilter::from(LogLevel::Info),
        node,
    )
    .await;

    Cli::<MyRethCliExt>::parse().run().unwrap();

    future::select_all(vec![threads]).await.0.unwrap();
}

/// The type that tells the reth CLI what extensions to use
struct MyRethCliExt;

impl RethCliExt for MyRethCliExt {
    /// This tells the reth CLI to install the `txpool` rpc namespace via `RethCliTxpoolExt`
    type Node = RethCliTxpoolExt;
}

/// Our custom cli args extension that adds one flag to reth default CLI.
#[derive(Debug, Clone, Copy, Default, clap::Args)]
struct RethCliTxpoolExt {
    /// CLI flag to enable the txpool extension namespace
    #[clap(long)]
    pub enable_ext: bool,
}

impl RethNodeCommandConfig for RethCliTxpoolExt {
    // This is the entrypoint for the CLI to extend the RPC server with custom rpc namespaces.
    fn extend_rpc_modules<Conf, Reth>(
        &mut self,
        _config: &Conf,
        _components: &Reth,
        rpc_components: RethRpcComponents<'_, Reth>,
    ) -> eyre::Result<()>
    where
        Conf: RethRpcConfig,
        Reth: RethNodeComponents,
    {
        if !self.enable_ext {
            return Ok(());
        }

        // here we get the configured pool type from the CLI.
        let pool = rpc_components.registry.pool().clone();
        let ext = TxpoolExt { pool };

        // now we merge our extension namespace into all configured transports
        rpc_components.modules.merge_configured(ext.into_rpc())?;

        println!("txpool extension enabled");
        Ok(())
    }
}

/// trait interface for a custom rpc namespace: `txpool`
///
/// This defines an additional namespace where all methods are configured as trait functions.
#[cfg_attr(not(test), rpc(server, namespace = "txpoolExt"))]
#[cfg_attr(test, rpc(server, client, namespace = "txpoolExt"))]
pub trait TxpoolExtApi {
    /// Returns the number of transactions in the pool.
    #[method(name = "transactionCount")]
    fn transaction_count(&self) -> RpcResult<usize>;
}

/// The type that implements the `txpool` rpc namespace trait
pub struct TxpoolExt<Pool> {
    pool: Pool,
}

impl<Pool> TxpoolExtApiServer for TxpoolExt<Pool>
where
    Pool: TransactionPool + Clone + 'static,
{
    fn transaction_count(&self) -> RpcResult<usize> {
        Ok(self.pool.pool_size().total)
    }
}
