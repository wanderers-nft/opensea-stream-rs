use serde::{de::Error, Deserialize, Serialize};
use std::fmt::Display;
use url::Url;

/// A collection whose events can be subscribed to.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Collection {
    /// Collection with slug.
    Collection(String),
    /// All possible collections.
    All,
}

impl Display for Collection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "collection:{}",
            match &self {
                Collection::Collection(c) => c,
                Collection::All => "*",
            }
        )
    }
}

impl Serialize for Collection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Collection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        let s = s
            .strip_prefix("collection:")
            .ok_or_else(|| D::Error::custom("expected collection:name"))?;

        Ok(match s {
            "*" => Collection::All,
            _ => Collection::Collection(s.to_owned()),
        })
    }
}

/// The websocket to connect to.
///
/// OpenSea provides two websockets for either `Mainnet` (production) networks for `Testnet` networks.
/// See [`Chain`](crate::schema::Chain) for a full list of supported chains.
pub enum Network {
    /// Mainnet (`Ethereum`, `Polygon`, `Klaytn`, `Solana`)
    Mainnet,
    /// Testnet (`Goerli`, `Mumbai`, `Baobab`)
    Testnet,
}

impl From<Network> for Url {
    fn from(val: Network) -> Self {
        match val {
            Network::Mainnet => {
                Url::parse("wss://stream.openseabeta.com/socket/websocket").unwrap()
            }
            Network::Testnet => {
                Url::parse("wss://testnets-stream.openseabeta.com/socket/websocket").unwrap()
            }
        }
    }
}

/// Receivable events from the websocket.
///
/// This type belongs to the `event` field of [`Message`](phyllo::message::Message), not to be confused with
/// [`Payload`](crate::schema::Payload).
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Event {
    /// An item been listed for sale.
    ItemListed,
    /// An item has been sold.
    ItemSold,
    /// An item has been transferred from one wallet to another.
    ItemTransferred,
    /// An item has had its metadata updated.
    ItemMetadataUpdated,
    /// An item has had its listing cancelled.
    ItemCancelled,
    /// An item has received an offer.
    ItemReceivedOffer,
    /// An item has received a bid.
    ItemReceivedBid,
}
