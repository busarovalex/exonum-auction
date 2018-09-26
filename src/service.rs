use exonum::{
    api::ServiceApiBuilder, blockchain::{self, Transaction, TransactionSet}, crypto::Hash,
    encoding::Error as StreamStructError, helpers::fabric, messages::RawTransaction,
    storage::Snapshot,
};

use super::api::PublicApi;
use super::schema::Schema;
use super::transactions::AuctionTransactions;

pub const AUCTION_SERVICE: u16 = 1270;
const SERVICE_NAME: &str = "auction";

/// Exonum `Service` implementation.
#[derive(Debug, Default)]
pub struct Service;

impl blockchain::Service for Service {
    fn service_id(&self) -> u16 {
        AUCTION_SERVICE
    }

    fn service_name(&self) -> &'static str {
        SERVICE_NAME
    }

    fn state_hash(&self, view: &dyn Snapshot) -> Vec<Hash> {
        let schema = Schema::new(view);
        schema.state_hash()
    }

    fn tx_from_raw(&self, raw: RawTransaction) -> Result<Box<dyn Transaction>, StreamStructError> {
        let tx = AuctionTransactions::tx_from_raw(raw)?;
        Ok(tx.into())
    }

    fn wire_api(&self, builder: &mut ServiceApiBuilder) {
        PublicApi::wire(builder);
    }
}

/// A configuration service creator for the `NodeBuilder`.
#[derive(Debug, Clone, Copy)]
pub struct ServiceFactory;

impl fabric::ServiceFactory for ServiceFactory {
    fn service_name(&self) -> &str {
        SERVICE_NAME
    }

    fn make_service(&mut self, _: &fabric::Context) -> Box<dyn blockchain::Service> {
        Box::new(Service)
    }
}
