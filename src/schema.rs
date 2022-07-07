use crate::Event;
use chrono::{DateTime, Utc};
use ethers::{
    abi::Address,
    prelude::{H256, U256},
};
use serde::{de::Error, Deserialize, Serialize};
use std::{fmt, str::FromStr};
use url::Url;

/// Payload of a message received from the websocket.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StreamEvent {
    /// Timestamp of when this message was sent to the client.
    pub sent_at: DateTime<Utc>,
    /// Contents of the message
    #[serde(flatten)]
    pub payload: Payload,
}

/// Content of the message.
///
/// This type corresponds to the JSON objects recieved [as described here](https://docs.opensea.io/reference/stream-api-event-schemas),
/// not the event type used for the Phoenix protocol (see [`Event`]).
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "event_type", content = "payload")]
#[serde(rename_all = "snake_case")]
pub enum Payload {
    /// An item has been listed for sale.
    ItemListed(ItemListedData),
    /// An item has been sold.
    ItemSold(ItemSoldData),
    /// An item has been transferred from one wallet to another.
    ItemTransferred(ItemTransferredData),
    /// An item has had its metadata updated.
    ItemMetadataUpdated(ItemMetadataUpdatedData),
    /// An item has had its listing cancelled.
    ItemCancelled(ItemCancelledData),
    /// An item has received an offer.
    ItemReceivedOffer(ItemReceivedOfferData),
    /// An item has received a bid.
    ItemReceivedBid(ItemReceivedBidData),
}

impl From<Payload> for Event {
    fn from(val: Payload) -> Self {
        match val {
            Payload::ItemListed(_) => Event::ItemListed,
            Payload::ItemSold(_) => Event::ItemSold,
            Payload::ItemTransferred(_) => Event::ItemTransferred,
            Payload::ItemMetadataUpdated(_) => Event::ItemMetadataUpdated,
            Payload::ItemCancelled(_) => Event::ItemCancelled,
            Payload::ItemReceivedOffer(_) => Event::ItemReceivedOffer,
            Payload::ItemReceivedBid(_) => Event::ItemReceivedBid,
        }
    }
}

/// Context for a message (token and collection)
///
/// This struct is present in every [`Payload`].
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Context {
    /// Collection that the token belongs to.
    pub collection: Collection,
    /// Information about the item itself.
    pub item: Item,
}

/// A collection on OpenSea.
#[derive(Debug, Clone)]
pub struct Collection(String);

impl Serialize for Collection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct Inner {
            slug: String,
        }

        Inner {
            slug: self.0.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Collection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Inner {
            slug: String,
        }

        Deserialize::deserialize(deserializer).map(|v: Inner| Collection(v.slug))
    }
}

/// Context about an item.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    /// Identifier.
    pub nft_id: NftId,
    /// Link to OpenSea page.
    pub permalink: Url,
    /// Chain the item is on.
    pub chain: Chain,
    /// Basic metadata.
    pub metadata: Metadata,
}

/// Identifier of the NFT.
#[derive(Debug, Clone)]
pub struct NftId {
    /// Chain the item is on.
    pub network: Chain,
    /// Contract address.
    pub address: Address,
    /// Token ID.
    pub id: U256,
}

impl Serialize for NftId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        format!("{}/{:?}/{}", self.network, self.address, self.id).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for NftId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        let mut parts = s.splitn(3, '/').fuse();

        let network = parts
            .next()
            .map(Chain::from_str)
            .ok_or_else(|| D::Error::custom("expected network"))?
            .map_err(|_| D::Error::custom("invalid network"))?;

        let address = parts
            .next()
            .map(Address::from_str)
            .ok_or_else(|| D::Error::custom("expected address"))?
            .map_err(D::Error::custom)?;

        let id = parts
            .next()
            .map(U256::from_dec_str)
            .ok_or_else(|| D::Error::custom("expected id"))?
            .map_err(D::Error::custom)?;

        Ok(NftId {
            network,
            address,
            id,
        })
    }
}

/// Network an item is on.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(tag = "name", rename_all = "lowercase")]
#[non_exhaustive]
pub enum Chain {
    /// [Ethereum](https://ethereum.org) mainnet.
    Ethereum,
    /// [Polygon](https://polygon.technology/solutions/polygon-pos) mainnet.
    #[serde(rename = "matic")]
    Polygon,
    /// [Klaytn](https://www.klaytn.foundation/) mainnet.
    Klaytn,
    /// [Solana](https://solana.com/) mainnet. This variant (and all events for Solana assets) are not supported in this version.
    Solana,

    /// [Rinkeby](https://ethereum.org/en/developers/docs/networks/#rinkeby) testnet (of Ethereum).
    Rinkeby,
    /// [Mumbai](https://docs.polygon.technology/docs/develop/network-details/network#mumbai-pos-testnet) testnet (of Polygon).
    Mumbai,
    /// [Baobab](https://www.klaytn.foundation/) testnet (of Klaytn).
    Baobab,
}

impl FromStr for Chain {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ethereum" => Ok(Chain::Ethereum),
            "matic" => Ok(Chain::Polygon),
            "klaytn" => Ok(Chain::Klaytn),
            "solana" => Ok(Chain::Solana),
            "rinkeby" => Ok(Chain::Rinkeby),
            "mumbai" => Ok(Chain::Mumbai),
            "baobab" => Ok(Chain::Baobab),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Chain::Ethereum => "ethereum",
                Chain::Polygon => "matic",
                Chain::Klaytn => "klaytn",
                Chain::Solana => "solana",
                Chain::Rinkeby => "rinkeby",
                Chain::Mumbai => "mumbai",
                Chain::Baobab => "baobab",
            }
        )
    }
}

/// Basic metadata of an item.
///
/// This is fetched directly from an item's metadata according to [metadata standards](https://docs.opensea.io/docs/metadata-standards).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metadata {
    /// Name.
    pub name: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Image URL. This is shown on the collection's storefront.
    pub image_url: Option<Url>,
    /// Animation URL. This is shown on the item's page.
    pub animation_url: Option<Url>,
    /// URL to metadata.
    pub metadata_url: Option<Url>,
}

/// Payload data for [`Payload::ItemListed`].
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemListedData {
    /// Context
    #[serde(flatten)]
    pub context: Context,

    /// Timestamp of when the listing was created.
    pub event_timestamp: DateTime<Utc>,
    /// Starting price of the listing. See `payment_token` for the actual value of each unit.
    #[serde(with = "u256_fromstr_radix_10")]
    pub base_price: U256,
    /// Expiration date.
    pub expiration_date: DateTime<Utc>,
    /// Whether the listing is private.
    pub is_private: bool,
    /// Timestamp of when the listing was created.
    pub listing_date: DateTime<Utc>,
    /// Type of listing. `None` indicates the listing is a buyout.
    pub listing_type: Option<ListingType>,
    /// Creator of the listing.
    #[serde(with = "address_fromjson")]
    pub maker: Address,
    /// Token accepted for payment.
    pub payment_token: PaymentToken,
    /// Number of items on sale. This is always `1` for ERC-721 tokens.
    pub quantity: u64,
    /// Buyer of the listing.
    #[serde(with = "address_fromjson_opt", default)]
    pub taker: Option<Address>,
}

/// Payload data for [`Payload::ItemSold`].
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemSoldData {
    /// Context
    #[serde(flatten)]
    pub context: Context,

    /// Timestamp of when the item was sold.
    pub event_timestamp: DateTime<Utc>,
    /// Timestamp of when the listing was closed.
    pub closing_date: DateTime<Utc>,
    /// Whether the listing was private.
    pub is_private: bool,
    /// Type of listing. `None` indicates the listing was a buyout.
    pub listing_type: Option<ListingType>,
    /// Creator of the listing.
    #[serde(with = "address_fromjson")]
    pub maker: Address,
    /// Token used for payment.
    pub payment_token: PaymentToken,
    /// Number of items bought. This is always `1` for ERC-721 tokens.
    pub quantity: u64,
    /// Purchase price. See `payment_token` for the actual value of each unit.
    #[serde(with = "u256_fromstr_radix_10")]
    pub sale_price: U256,
    /// Buyer/winner of the listing.
    #[serde(with = "address_fromjson")]
    pub taker: Address,
    /// Transaction for the purchase.
    pub transaction: Transaction,
}

/// Payload data for [`Payload::ItemTransferred`].
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemTransferredData {
    /// Context
    #[serde(flatten)]
    pub context: Context,

    /// Timestamp of when the item was transferred.
    pub event_timestamp: DateTime<Utc>,
    /// Transaction of the transfer.
    pub transaction: Transaction,
    /// Address the item was transferred from.
    #[serde(with = "address_fromjson")]
    pub from_account: Address,
    /// Address the item was transferred to.
    #[serde(with = "address_fromjson")]
    pub to_account: Address,
    /// Number of items transferred. This is always `1` for ERC-721 tokens.
    pub quantity: u64,
}

/// Payload data for [`Payload::ItemMetadataUpdated`].
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemMetadataUpdatedData {
    /// Context
    #[serde(flatten)]
    pub context: Context,

    /// New name.
    pub name: Option<String>,
    /// New description.
    pub description: Option<String>,
    /// New cached preview URL.
    pub image_preview_url: Option<Url>,
    /// New animation URL.
    pub animation_url: Option<Url>,
    /// New background color.
    pub background_color: Option<String>,
    /// New URL to metadata
    pub metadata_url: Option<Url>,
    /// TODO: what's here?
    #[serde(default)]
    pub traits: Vec<serde_json::Value>,
}

/// Payload data for [`Payload::ItemCancelled`].
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemCancelledData {
    /// Context
    #[serde(flatten)]
    pub context: Context,

    /// Timestamp of when the listing was cancelled.
    pub event_timestamp: DateTime<Utc>,
    /// Type of listing. `None` indicates the listing would've been a buyout.
    pub listing_type: Option<ListingType>,
    /// Token accepted for payment.
    pub payment_token: PaymentToken,
    /// Number of items in listing. This is always `1` for ERC-721 tokens.
    pub quantity: u64,
    /// Transaction for the cancellation.
    pub transaction: Transaction,
}

/// Payload data for [`Payload::ItemReceivedOffer`].
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemReceivedOfferData {
    /// Context
    #[serde(flatten)]
    pub context: Context,

    /// Timestamp of when the offer was received.
    pub event_timestamp: DateTime<Utc>,
    /// Offer price. See `payment_token` for the actual value of each unit.
    #[serde(with = "u256_fromstr_radix_10")]
    pub base_price: U256,
    /// Timestamp of when the offer was created.
    pub created_date: DateTime<Utc>,
    /// Timestamp of when the offer will expire.
    pub expiration_date: DateTime<Utc>,
    /// Creator of the offer.
    #[serde(with = "address_fromjson")]
    pub maker: Address,
    /// Token offered for payment.
    pub payment_token: PaymentToken,
    /// Number of items on the offer. This is always `1` for ERC-721 tokens.
    pub quantity: u64,
    /// Taker of the offer.
    #[serde(with = "address_fromjson_opt", default)]
    pub taker: Option<Address>,
}

/// Payload data for [`Payload::ItemReceivedBid`].
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemReceivedBidData {
    /// Context
    #[serde(flatten)]
    pub context: Context,

    /// Timestamp of when the bid was received.
    pub event_timestamp: DateTime<Utc>,
    /// Bid price. See `payment_token` for the actual value of each unit.
    #[serde(with = "u256_fromstr_radix_10")]
    pub base_price: U256,
    /// Timestamp of when the bid was created.
    pub created_date: DateTime<Utc>,
    /// Timestamp of when the bid will expire.
    pub expiration_date: DateTime<Utc>,
    /// Creator of the bid.
    #[serde(with = "address_fromjson")]
    pub maker: Address,
    /// Token offered for payment.
    pub payment_token: PaymentToken,
    /// Number of items on the offer. This is always `1` for ERC-721 tokens.
    pub quantity: u64,
    /// Taker of the bid.
    #[serde(with = "address_fromjson_opt", default)]
    pub taker: Option<Address>,
}

/// Auctioning system used by the listing.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ListingType {
    /// [English](https://en.wikipedia.org/wiki/English_auction) (ascending).
    English,
    /// [Dutch](https://en.wikipedia.org/wiki/Dutch_auction) (descending).
    Dutch,
}

impl fmt::Display for ListingType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ListingType::English => "English",
                ListingType::Dutch => "Dutch",
            }
        )
    }
}

mod address_fromjson {
    use ethers::abi::Address;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    struct Inner {
        address: Address,
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Address, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer).map(|v: Inner| v.address)
    }

    pub fn serialize<S>(value: &Address, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Inner { address: *value }.serialize(serializer)
    }
}

mod address_fromjson_opt {
    use ethers::abi::Address;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    struct Inner {
        address: Address,
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Address>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner: Option<Inner> = Deserialize::deserialize(deserializer)?;
        Ok(inner.map(|i| i.address))
    }

    pub fn serialize<S>(value: &Option<Address>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value.map(|v| Inner { address: v }).serialize(serializer)
    }
}

/// Details of a transaction
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    /// Transaction hash
    pub hash: H256,
    /// Timestamp of transaction
    pub timestamp: DateTime<Utc>,
}

/// Token used for payment.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaymentToken {
    /// Contract address
    pub address: Address,
    /// Granularity of the token
    pub decimals: u64,
    /// Price of token (denominated in ETH)
    #[serde(with = "f64_fromstring")]
    pub eth_price: f64,
    /// Name
    pub name: String,
    /// Symbol
    pub symbol: String,
    /// Price of token (denominated in USD)
    #[serde(with = "f64_fromstring")]
    pub usd_price: f64,
}

// h/t: meetmangukiya (https://gist.github.com/meetmangukiya/40cad17bcb7d3196d33b072a3500fac7)
mod u256_fromstr_radix_10 {
    use super::*;
    use serde::{de::Visitor, Deserializer, Serializer};
    use std::fmt;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<U256, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Helper;

        impl<'de> Visitor<'de> for Helper {
            type Value = U256;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                U256::from_dec_str(value).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_str(Helper)
    }

    pub fn serialize<S>(value: &U256, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(&value)
    }
}

mod f64_fromstring {
    use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringFloat {
            Str(String),
            F64(f64),
        }

        match StringFloat::deserialize(deserializer)? {
            StringFloat::Str(s) => s.parse().map_err(D::Error::custom),
            StringFloat::F64(f) => Ok(f),
        }
    }

    pub fn serialize<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value.to_string().serialize(serializer)
    }
}
