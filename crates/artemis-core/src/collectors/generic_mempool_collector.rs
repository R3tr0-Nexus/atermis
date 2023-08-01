use async_trait::async_trait;

use ethers::{prelude::Middleware, types::Transaction};
use futures::stream::{iter, StreamExt};
use std::sync::Arc;



use crate::types::{Collector, CollectorStream};
use anyhow::Result;

/// A collector that listens for new transactions in the mempool, and generates a stream of
/// [events](Transaction) which contain the transaction.
pub struct GenericMempoolCollector<M> {
    
    provider: Arc<M>,
}

impl<M> GenericMempoolCollector<M> {
    pub fn new(provider: Arc<M>) -> Self {
        Self { provider }
    }
}

/// Implementation of the [Collector](Collector) trait for the [GenericMempoolCollector](GenericMempoolCollector).
/// This implementation uses the [PubsubClient](PubsubClient) to subscribe to new transactions.
#[async_trait]
impl<M> Collector<Transaction> for GenericMempoolCollector<M>
where
    M: Middleware,
    M::Error: 'static,
    
{
    async fn get_event_stream(&self) -> Result<CollectorStream<'_, Transaction>> {
        let stream = self.provider.txpool_content()
                                                                .await
                                                                .map_err(|_| anyhow::anyhow!("Failed to create mempool stream"))?;

        let mut pending_txs = Vec::new();

        let _z: () = stream.pending.into_values().
                     map( |tx_treemap| {
                                                        
                         let txs: Vec<Transaction> = tx_treemap.into_values()
                            .map(|tx| {
                                                                                            
                                    tx
                                                        
                            }) 
                            .collect();
                                                        
                    pending_txs.push(txs);
                                                        
                    }).collect();
                                                        
        let pending_txs: Vec<Transaction> = pending_txs.into_iter().flatten().collect();

        let pending_tx = iter(pending_txs).boxed();
        



        Ok(pending_tx)

    }
}