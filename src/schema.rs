use exonum::{
    crypto::{Hash, PublicKey, Signature}, 
    storage::{Fork, ProofMapIndex, Snapshot}
};

encoding_struct! {
    /// Информация об аукционе, хранится в базе данных
    struct Auction {
        /// Хозяин аукциона
        owner: &PublicKey,
        /// Текущий список ставок по лоту
        bids: Vec<Bid>
    }
}

encoding_struct! {
    /// Информация о претенденте на лот
    struct Bid {
        /// Публичный ключ претендента
        bidder: &PublicKey,
        /// Величина ставки претендента
        value: u64,
    }
}

encoding_struct! {
    /// Информация о закрытом аукционе, хранится в базе данных
    struct ClosedAuction {
        /// Хозяин закрытого аукциона
        owner: &PublicKey,
        /// Текущий список ставок по лоту
        bids: Vec<SecretBid>
    }
}

encoding_struct! {
    /// Информация о претенденте на лот
    struct SecretBid {
        /// Публичный ключ претендента
        bidder: &PublicKey,
        /// Подписанная претендентом величина ставки
        value_signature: &Signature,
        /// Величина ставки претендента
        value: u64
    }
}

/// Database schema for the cryptocurrency.
#[derive(Debug)]
pub struct Schema<T> {
    view: T,
}

impl<T> Schema<T> {
    /// Creates a new schema from the database view.
    pub fn new(snapshot: T) -> Self {
        Schema { view: snapshot }
    }
}

impl<T> Schema<T>
where
    T: AsRef<dyn Snapshot>,
{
    /// Returns the `ProofMapIndex` of auctions.
    pub fn auctions(&self) -> ProofMapIndex<&T, PublicKey, Auction> {
        ProofMapIndex::new("auction.auctions", &self.view)
    }
    /// Returns the `ProofMapIndex` of lots.
    pub fn closed_auctions(&self) -> ProofMapIndex<&T, PublicKey, ClosedAuction> {
        ProofMapIndex::new("auction.closed_auctions", &self.view)
    }

    /// Returns the state hash of the timestamping service.
    pub fn state_hash(&self) -> Vec<Hash> {
        vec![self.auctions().merkle_root()]
    }
}

impl<'a> Schema<&'a mut Fork> {
    /// Returns the mutable `ProofMapIndex` of auctions.
    pub fn auctions_mut(&mut self) -> ProofMapIndex<&mut Fork, PublicKey, Auction> {
        ProofMapIndex::new("auction.auctions", &mut self.view)
    }

    /// Returns the mutable `ProofMapIndex` of closed auctions.
    pub fn closed_auctions_mut(&mut  self) -> ProofMapIndex<&mut Fork, PublicKey, ClosedAuction> {
        ProofMapIndex::new("auction.closed_auctions", &mut self.view)
    }

    /// Adds the auction entry to the database.
    pub fn add_auction(&mut self, new_auction: Auction) {
        let owner = new_auction.owner();
        if self.auctions().contains(owner) {
            return;
        }
        self.auctions_mut().put(owner, new_auction.clone());
    }

    /// Добавляет претендента на аукцион, если такого еще нет
    pub fn bid(&mut self, owner: &PublicKey, new_bid: Bid) {
        if let Some(auction) = self.auctions().get(&owner).clone() {
            if auction.bids().iter().find(|bid| bid.bidder() == new_bid.bidder()).is_some() {
                return;
            }
            auction.bids().push(new_bid);
            self.auctions_mut().put(&owner, auction);
        }
    }

    /// Adds the auction entry to the database.
    pub fn add_closed_auction(&mut self, new_auction: ClosedAuction) {
        let owner = new_auction.owner();

        if self.closed_auctions().contains(owner) {
            return;
        }

        self.closed_auctions_mut().put(owner, new_auction.clone());
    }

    /// Добавляет претендента на закрытый аукцион или заменяет существующий
    pub fn secret_bid(&mut self, owner: &PublicKey, new_bid: SecretBid) {
        if let Some(auction) = self.closed_auctions().get(&owner).clone() {
            auction.bids().retain(|bid| bid.bidder() != new_bid.bidder());
            auction.bids().push(new_bid);
            self.closed_auctions_mut().put(&owner, auction);
        }
    }
}
