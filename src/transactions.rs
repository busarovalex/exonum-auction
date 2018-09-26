use exonum::{
    blockchain::{ExecutionError, ExecutionResult, Transaction}, 
    crypto::{self, PublicKey, Signature},
    messages::Message, storage::Fork,
};

use super::schema::{Schema, Lot, Bid, ClosedAuction, SecretBid};
use super::service::AUCTION_SERVICE;

#[derive(Debug, Fail)]
#[repr(u8)]
pub enum Error {
    #[fail(display = "Public key already owns a lot")]
    LotAlreadyExists = 0,
    #[fail(display = "No lot with such public key")]
    LotDoesNotExists = 1,
    #[fail(display = "Secret bid verification failed")]
    SecretBidVerificationFailed = 2,
}

impl From<Error> for ExecutionError {
    fn from(value: Error) -> ExecutionError {
        let description = value.to_string();
        ExecutionError::with_description(value as u8, description)
    }
}

transactions! {
    /// Transaction group.
    pub AuctionTransactions {
        const SERVICE_ID = AUCTION_SERVICE;

        struct TxCreateLot {
            /// Public key of transaction.
            pub_key: &PublicKey,
        }

        struct TxBid {
            /// Public key of transaction.
            pub_key: &PublicKey,
            lot_owner: &PublicKey,
            value: u64
        }

        struct TxCreateClosedAuction {
            /// Public key of transaction.
            pub_key: &PublicKey,
        }

        struct TxSecretBid {
            /// Public key of transaction.
            pub_key: &PublicKey,
            lot_owner: &PublicKey,
            value_signature: &Signature
        }

        struct TxProveSecretBid {
            /// Public key of transaction.
            pub_key: &PublicKey,
            lot_owner: &PublicKey,
            value: u64
        }
    }
}

impl Transaction for TxCreateLot {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let owner = self.pub_key();

        let mut schema = Schema::new(fork);
        if schema.lots().get(owner).is_some() {
            Err(Error::LotAlreadyExists)?;
        }

        let entry = Lot::new(owner, Vec::new());
        schema.add_lot(entry);
        trace!("TxCreateLot executed");
        Ok(())
    }
}

impl Transaction for TxBid {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let bidder = self.pub_key();
        let lot_owner = self.lot_owner();
        let mut schema = Schema::new(fork);
        if schema.lots().get(lot_owner).is_none() {
            Err(Error::LotDoesNotExists)?;
        }

        let entry = Bid::new(bidder, self.value());
        schema.bid(lot_owner, entry);
        trace!("TxBid executed");
        Ok(())
    }
}

impl Transaction for TxCreateClosedAuction {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let owner = self.pub_key();

        let mut schema = Schema::new(fork);
        if schema.closed_auctions().get(owner).is_some() {
            Err(Error::LotAlreadyExists)?;
        }

        let entry = ClosedAuction::new(owner, Vec::new());
        schema.add_closed_auction(entry);
        trace!("TxCreateClosedAuction executed");
        Ok(())
    }
}

impl Transaction for TxSecretBid {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let bidder = self.pub_key();
        let lot_owner = self.lot_owner();
        let mut schema = Schema::new(fork);
        if schema.lots().get(lot_owner).is_none() {
            Err(Error::LotDoesNotExists)?;
        }

        let entry = SecretBid::new(bidder, self.value_signature(), 0);
        schema.secret_bid(lot_owner, entry);
        trace!("TxSecretBid executed");
        Ok(())
    }
}

impl Transaction for TxProveSecretBid {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let bidder = self.pub_key();
        let lot_owner = self.lot_owner();
        let mut schema = Schema::new(fork);
        let auction = schema.closed_auctions().get(lot_owner).ok_or(Error::LotDoesNotExists)?;
        let bids = auction.bids();
        let bid = bids.iter().find(|bid| bid.bidder() == bidder).ok_or(Error::LotDoesNotExists)?;
        let encoded_value: Vec<u8> = ::bincode::serialize(&self.value()).unwrap();

        if crypto::verify(bid.value_signature(), &encoded_value, bidder) {
            let entry = SecretBid::new(bidder, bid.value_signature(), self.value());
            schema.secret_bid(lot_owner, entry);
        } else {
            Err(Error::SecretBidVerificationFailed)?;
        }

        trace!("TxProveSecretBid executed");
        Ok(())
    }
}
