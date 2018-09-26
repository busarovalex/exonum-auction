use exonum::{
    blockchain::{ExecutionError, ExecutionResult, Transaction}, 
    crypto::{self, PublicKey, Signature},
    messages::Message, storage::Fork,
};

use super::schema::{Schema, Auction, Bid, ClosedAuction, SecretBid};
use super::service::AUCTION_SERVICE;

#[derive(Debug, Fail)]
#[repr(u8)]
pub enum Error {
    #[fail(display = "Public key already owns an auction")]
    AuctionAlreadyExists = 0,
    #[fail(display = "No lot with such public key")]
    AuctionDoesNotExists = 1,
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

        /// Создает новый аукцион
        struct TxCreateAuction {
            /// Public key of transaction.
            pub_key: &PublicKey,
        }

        /// Делает ставку на аукционе
        struct TxBid {
            /// Public key of transaction.
            pub_key: &PublicKey,
            /// Публичный ключ хозяина аукциона
            auction_owner: &PublicKey,
            /// Величина ставки
            value: u64
        }

        /// Создает новый закрытый аукцион
        struct TxCreateClosedAuction {
            /// Public key of transaction.
            pub_key: &PublicKey,
        }

        /// Делает ставку на закрытом аукционе
        struct TxSecretBid {
            /// Public key of transaction.
            pub_key: &PublicKey,
            /// Публичный ключ хозяина аукциона
            auction_owner: &PublicKey,
            /// Подписанная величина ставки
            value_signature: &Signature
        }

        /// Подтверждает ставку на закрытом аукционе после окончания периода торга
        struct TxProveSecretBid {
            /// Public key of transaction.
            pub_key: &PublicKey,
            /// Публичный ключ хозяина аукциона
            auction_owner: &PublicKey,
            /// Величина ставки
            value: u64
        }
    }
}

impl Transaction for TxCreateAuction {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let owner = self.pub_key();

        let mut schema = Schema::new(fork);
        if schema.auctions().get(owner).is_some() {
            Err(Error::AuctionAlreadyExists)?;
        }

        let entry = Auction::new(owner, Vec::new());
        schema.add_auction(entry);
        trace!("TxCreateAuction executed");
        Ok(())
    }
}

impl Transaction for TxBid {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let bidder = self.pub_key();
        let auction_owner = self.auction_owner();
        let mut schema = Schema::new(fork);
        if schema.auctions().get(auction_owner).is_none() {
            Err(Error::AuctionDoesNotExists)?;
        }

        let entry = Bid::new(bidder, self.value());
        schema.bid(auction_owner, entry);
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
            Err(Error::AuctionAlreadyExists)?;
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
        let auction_owner = self.auction_owner();
        let mut schema = Schema::new(fork);
        if schema.auctions().get(auction_owner).is_none() {
            Err(Error::AuctionDoesNotExists)?;
        }

        let entry = SecretBid::new(bidder, self.value_signature(), 0);
        schema.secret_bid(auction_owner, entry);
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
        let auction_owner = self.auction_owner();
        let mut schema = Schema::new(fork);
        let auction = schema.closed_auctions().get(auction_owner).ok_or(Error::AuctionDoesNotExists)?;
        let bids = auction.bids();
        let bid = bids.iter().find(|bid| bid.bidder() == bidder).ok_or(Error::AuctionDoesNotExists)?;
        let encoded_value: Vec<u8> = ::bincode::serialize(&self.value()).unwrap();

        if crypto::verify(bid.value_signature(), &encoded_value, bidder) {
            let entry = SecretBid::new(bidder, bid.value_signature(), self.value());
            schema.secret_bid(auction_owner, entry);
        } else {
            Err(Error::SecretBidVerificationFailed)?;
        }

        trace!("TxProveSecretBid executed");
        Ok(())
    }
}
