use exonum::{
    api::{self, ServiceApiBuilder, ServiceApiState},
    blockchain::{Transaction}, crypto::{Hash, PublicKey},
    node::TransactionSend
};

use super::transactions::AuctionTransactions;
use super::schema::{Schema, Auction};

/// Описывает параметры для 'handle_lot' Endpoint'а
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct LotQuery {
    pub owner: PublicKey,   
}

/// Response to an incoming transaction returned by the REST API.
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    /// Hash of the transaction.
    pub tx_hash: Hash,
}

/// Public service API description.
#[derive(Debug, Clone, Copy)]
pub struct PublicApi;

impl PublicApi {
    /// Endpoint для получения информации по лоту аукциона
    pub fn handle_lot(state: &ServiceApiState,
        query: LotQuery,) ->  api::Result<Option<Auction>> {
        let snapshot = state.snapshot();
        let schema = Schema::new(&snapshot);
        Ok(schema.auctions().get(&query.owner))
    }

    /// Endpoint for handling transactions.
    pub fn post_transaction(
        state: &ServiceApiState,
        query: AuctionTransactions,
    ) -> api::Result<TransactionResponse> {
        let transaction: Box<dyn Transaction> = query.into();
        let tx_hash = transaction.hash();
        state.sender().send(transaction)?;
        Ok(TransactionResponse { tx_hash })
    }

    /// Wires the above endpoint to public scope of the given `ServiceApiBuilder`.
    pub fn wire(builder: &mut ServiceApiBuilder) {
        builder
            .public_scope()
            .endpoint("v1/lots/lot", Self::handle_lot)
            .endpoint_mut("v1/lots/transaction", Self::post_transaction);
    }
}
