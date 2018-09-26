use exonum::{
    crypto::{Hash, PublicKey, Signature}, 
    storage::{Fork, ProofMapIndex, Snapshot}
};

encoding_struct! {
    struct Bid {
        bidder: &PublicKey,
        value: u64,
    }
}

encoding_struct! {
    struct Lot {
        owner: &PublicKey,
        bids: Vec<Bid>
    }
}

encoding_struct! {
    struct ClosedAuction {
        owner: &PublicKey,
        bids: Vec<SecretBid>
    }
}

encoding_struct! {
    struct SecretBid {
        bidder: &PublicKey,
        value_signature: &Signature,
        value: u64
    }
}

#[derive(Debug)]
pub struct Schema<T> {
    view: T,
}

impl<T> Schema<T> {
    pub fn new(snapshot: T) -> Self {
        Schema { view: snapshot }
    }
}

impl<T> Schema<T>
where
    T: AsRef<dyn Snapshot>,
{
    /// Returns the `ProofMapIndex` of lots.
    pub fn lots(&self) -> ProofMapIndex<&T, PublicKey, Lot> {
        ProofMapIndex::new("auction.lots", &self.view)
    }
    /// Returns the `ProofMapIndex` of lots.
    pub fn closed_auctions(&self) -> ProofMapIndex<&T, PublicKey, ClosedAuction> {
        ProofMapIndex::new("auction.closed_auctions", &self.view)
    }

    /// Returns the state hash of the timestamping service.
    pub fn state_hash(&self) -> Vec<Hash> {
        vec![self.lots().merkle_root()]
    }
}

impl<'a> Schema<&'a mut Fork> {
    /// Returns the mutable `ProofMapIndex` of timestamps.
    pub fn lots_mut(&mut self) -> ProofMapIndex<&mut Fork, PublicKey, Lot> {
        ProofMapIndex::new("auction.lots", &mut self.view)
    }

    pub fn closed_auctions_mut(&mut  self) -> ProofMapIndex<&mut Fork, PublicKey, ClosedAuction> {
        ProofMapIndex::new("auction.closed_auctions", &mut self.view)
    }

    /// Adds the lot entry to the database.
    pub fn add_lot(&mut self, new_lot: Lot) {
        let owner = new_lot.owner();

        // Check that lot with given owner does not exist.
        if self.lots().contains(owner) {
            return;
        }

        // Add lot
        self.lots_mut().put(owner, new_lot.clone());
    }

    pub fn bid(&mut self, owner: &PublicKey, new_bid: Bid) {
        if let Some(lot) = self.lots().get(&owner).clone() {
            if lot.bids().iter().find(|bid| bid.bidder() == new_bid.bidder()).is_some() {
                return;
            }
            lot.bids().push(new_bid);
            self.lots_mut().put(&owner, lot);
        }
    }

    pub fn add_closed_auction(&mut self, new_auction: ClosedAuction) {
        let owner = new_auction.owner();

        if self.closed_auctions().contains(owner) {
            return;
        }

        self.closed_auctions_mut().put(owner, new_auction.clone());
    }

    pub fn secret_bid(&mut self, owner: &PublicKey, new_bid: SecretBid) {
        if let Some(auction) = self.closed_auctions().get(&owner).clone() {
            auction.bids().retain(|bid| bid.bidder() != new_bid.bidder());
            auction.bids().push(new_bid);
            self.closed_auctions_mut().put(&owner, auction);
        }
    }
}
