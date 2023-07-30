use std::sync::Arc;

use crate::types::Executor;
use anyhow::Result;
use async_trait::async_trait;
use ethers::{signers::Signer, types::Chain};
use futures::{stream, StreamExt};
use matchmaker::{client::Client, types::BundleRequest};
use tracing::{error, info};

/// An executor that sends bundles to the MEV-share Matchmaker.
pub struct MevshareExecutor<S> {
    matchmaker_client: Client<S>,
}

/// List of bundles to send to the Matchmaker.
pub type Bundles = Vec<BundleRequest>;

impl<S: Signer + Clone + 'static> MevshareExecutor<S> {
    pub fn new(signer: S, chain: Chain, url: &str, relay_name: &str) -> Self {
        Self {
            matchmaker_client: Client::new(signer, chain, url, relay_name),
        }
    }
}

#[async_trait]
impl<S: Signer + Clone + 'static> Executor<Bundles> for MevshareExecutor<S> {
    /// Send bundles to the matchmaker.
    async fn execute(&self, action: Bundles) -> Result<()> {
        let bodies = stream::iter(action)
            .map(|bundle| {
                let client = &self.matchmaker_client;
                async move { client.send_bundle(&bundle).await }
            })
            .buffer_unordered(5);

        bodies
            .for_each(|b| async {
                match b {
                    Ok(b) => info!("Bundle response: {:?}", b),
                    Err(e) => error!("Bundle error: {}", e),
                }
            })
            .await;
        Ok(())
    }
}


pub async fn get_all_mev_share_endpoints<S: Signer + Clone + 'static>(tx_signer: S, chain: Chain) -> Vec<Arc<Box<MevshareExecutor<S>>>> {
    
    let endpoints = vec![
        ("flashbots", "https://relay.flashbots.net/"),
        ("builder0x69", "http://builder0x69.io/"),
        ("edennetwork", "https://api.edennetwork.io/v1/bundle"),
        ("beaverbuild", "https://rpc.beaverbuild.org/"),
        ("lightspeedbuilder", "https://rpc.lightspeedbuilder.info/"),
        ("eth-builder", "https://eth-builder.com/"),
        ("ultrasound", "https://relay.ultrasound.money/"),
        ("agnostic-relay", "https://agnostic-relay.net/"),
        ("relayoor-wtf", "https://relayooor.wtf/"),
        ("rsync-builder", "https://rsync-builder.xyz/"),
    ];

    let mut relays: Vec<Arc<Box<MevshareExecutor<S>>>> = vec![];

    for (name, endpoint) in endpoints {
        let relay = Arc::new(Box::new(MevshareExecutor::new(tx_signer.clone(), chain, endpoint, name)));
        relays.push(relay);
    }

    relays
}