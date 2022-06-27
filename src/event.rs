use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    ItemListed,
    ItemSold,
    ItemTransferred,
    ItemMetadataUpdated,
    ItemCancelled,
    ItemReceivedOffer,
    ItemReceivedBid,
}
