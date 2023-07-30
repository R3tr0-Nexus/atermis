use std::sync::Arc;

use anyhow::Result;
use artemis_core::{
    collectors::mevshare_collector::MevShareCollector,
    engine::Engine,
    executors::mev_share_executor::{MevshareExecutor, self},
    executors::flashbots_executor::{FlashbotsExecutor, self},
    types::{CollectorMap, ExecutorMap},
};
use clap::Parser;
use ethers::{
    prelude::MiddlewareBuilder,
    providers::{Provider, Ws},
    signers::{LocalWallet, Signer},
    types::{Address, Chain},
};
use mev_share_uni_arb::{
    strategy::MevShareUniArb,
    types::{Action, Event},
};
use tracing::{info, Level};
use tracing_subscriber::{filter, prelude::*};

/// CLI Options.
#[derive(Parser, Debug)]
pub struct Args {
    /// Ethereum node WS endpoint.
    #[arg(long)]
    pub wss: String,
    /// Private key for sending txs.
    #[arg(long)]
    pub private_key: String,
    /// MEV share signer
    #[arg(long)]
    pub flashbots_signer: String,
    /// Address of the arb contract.
    #[arg(long)]
    pub arb_contract_address: Address,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Set up tracing and parse args.
    let filter = filter::Targets::new()
        .with_target("mev_share_uni_arb", Level::INFO)
        .with_target("artemis_core", Level::INFO);
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    let args = Args::parse();

    //  Set up providers and signers.
    let ws = Ws::connect(args.wss).await?;
    let provider = Provider::new(ws);

    let wallet: LocalWallet = args.private_key.parse().unwrap();
    let address = wallet.address();

    let provider = Arc::new(provider.nonce_manager(address).with_signer(wallet.clone()));
    let fb_signer: LocalWallet = args.flashbots_signer.parse().unwrap();

    // Set up engine.
    let mut engine: Arc<Engine<Event, Action>> = Arc::new(Engine::default());

    // Set up collector.
    let mevshare_collector = Box::new(MevShareCollector::new(String::from(
        "https://mev-share.flashbots.net",
    )));
    let mevshare_collector = CollectorMap::new(mevshare_collector, Event::MEVShareEvent);
    let mut engine_ref = Arc::get_mut(&mut engine).unwrap();
    engine_ref.add_collector(Box::new(mevshare_collector));
    drop(engine_ref);


    // Set up strategy.
    let strategy = MevShareUniArb::new(
        Arc::new(provider.clone()),
        wallet.clone(),
        args.arb_contract_address,
    );
    let mut engine_ref = Arc::get_mut(&mut engine).unwrap();
    engine_ref.add_strategy(Box::new(strategy));
    drop(engine_ref);

    //Set up concurrent executors
    let mev_share_executors = mev_share_executor::get_all_mev_share_endpoints(fb_signer, Chain::Mainnet).await;

    for relay in mev_share_executors.into_iter()
    {   
        let engine = engine.clone();

        tokio::spawn(async move {

            let mut engine_clone = engine.clone();

            let mev_share_executor = Arc::into_inner(relay).unwrap();

            let mev_share_executor = ExecutorMap::new(mev_share_executor, |action| match action
            {
                Action::SubmitBundles(bundles) => Some(bundles),
            });
            let engine_ref = Arc::get_mut(&mut engine_clone).unwrap();
            engine_ref.add_executor(Box::new(mev_share_executor));
            drop(engine_ref);
        });
    }


    // Set up executor
    /*let mev_share_executor = Box::new(MevshareExecutor::new(fb_signer, Chain::Mainnet));
    let mev_share_executor = ExecutorMap::new(mev_share_executor, |action| match action {
        Action::SubmitBundles(bundles) => Some(bundles),
    });*/
    
    let engine = Arc::into_inner(engine).unwrap();

    // Start engine.
    if let Ok(mut set) = engine.run().await {
        while let Some(res) = set.join_next().await {
            info!("res: {:?}", res);
        }
    }

    Ok(())
}
