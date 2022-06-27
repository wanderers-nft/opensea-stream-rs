use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use ethers::prelude::{H256, U256};
use serde::de::Error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StreamEvent {
    pub sent_at: DateTime<Utc>,
    #[serde(flatten)]
    pub payload: Payload,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "event_type", content = "payload")]
#[serde(rename_all = "snake_case")]
pub enum Payload {
    ItemListed(ItemListedData),
    ItemSold(ItemSoldData),
    ItemTransferred(ItemTransferredData),
    ItemMetadataUpdated(ItemMetadataUpdatedData),
    ItemCancelled(ItemCancelledData),
    ItemReceivedOffer(ItemReceivedOfferData),
    ItemReceivedBid(ItemReceivedBidData),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Context {
    pub collection: Collection,
    pub item: Item,
}

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    pub nft_id: NftId,
    pub permalink: String,
    pub chain: Chain,
    pub metadata: Metadata,
}

#[derive(Debug, Clone)]
pub struct NftId {
    pub network: Chain,
    pub address: Address,
    pub id: U256,
}

impl Serialize for NftId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        format!("{}/{}/{}", self.network, self.address.0, self.id).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for NftId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        let mut parts = s.splitn(3, '/').fuse();

        let network = parts
            .next()
            .map(Chain::from_str)
            .ok_or_else(|| D::Error::custom("expected network"))?
            .map_err(|_| D::Error::custom("invalid network"))?;

        let address = parts
            .next()
            .map(|s| ethers::abi::Address::from_str(s).map(Address))
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(tag = "name", rename_all = "lowercase")]
pub enum Chain {
    Ethereum,
    #[serde(rename = "matic")]
    Polygon,
    Klaytn,
    Solana,

    Rinkeby,
    Mumbai,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metadata {
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub animation_url: Option<String>,
    pub metadata_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemListedData {
    #[serde(flatten)]
    pub context: Context,

    pub event_timestamp: DateTime<Utc>,
    #[serde(with = "u256_fromstr_radix_10")]
    pub base_price: U256,
    pub expiration_date: DateTime<Utc>,
    pub is_private: bool,
    pub listing_date: DateTime<Utc>,
    pub listing_type: ListingType,
    pub maker: Address,
    pub payment_token: PaymentToken,
    pub quantity: u64,
    pub taker: Option<Address>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemSoldData {
    #[serde(flatten)]
    pub context: Context,

    pub event_timestamp: DateTime<Utc>,
    pub closing_date: DateTime<Utc>,
    pub is_private: bool,
    pub listing_type: Option<ListingType>,
    pub maker: Address,
    pub payment_token: PaymentToken,
    pub quantity: u64,
    #[serde(with = "u256_fromstr_radix_10")]
    pub sale_price: U256,
    pub taker: Address,
    pub transaction: Transaction,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemTransferredData {
    #[serde(flatten)]
    pub context: Context,

    pub event_timestamp: DateTime<Utc>,
    pub transaction: Transaction,
    pub from_account: Address,
    pub to_account: Address,
    pub quantity: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemMetadataUpdatedData {
    #[serde(flatten)]
    pub context: Context,

    pub name: String,
    pub description: Option<String>,
    pub image_preview_url: Option<String>,
    pub animation_url: Option<String>,
    pub background_color: String,
    pub metadata_url: String,
    // todo: what's here?
    pub traits: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemCancelledData {
    #[serde(flatten)]
    pub context: Context,

    pub event_timestamp: DateTime<Utc>,
    pub listing_type: Option<ListingType>,
    pub payment_token: PaymentToken,
    pub quantity: u64,
    pub transaction: Transaction,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemReceivedOfferData {
    #[serde(flatten)]
    pub context: Context,

    pub event_timestamp: DateTime<Utc>,
    #[serde(with = "u256_fromstr_radix_10")]
    pub base_price: U256,
    pub created_date: DateTime<Utc>,
    pub expiration_date: DateTime<Utc>,
    pub maker: Address,
    pub payment_token: PaymentToken,
    pub quantity: u64,
    pub taker: Option<Address>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemReceivedBidData {
    #[serde(flatten)]
    pub context: Context,

    pub event_timestamp: DateTime<Utc>,
    #[serde(with = "u256_fromstr_radix_10")]
    pub base_price: U256,
    pub created_date: DateTime<Utc>,
    pub expiration_date: DateTime<Utc>,
    pub maker: Address,
    pub payment_token: PaymentToken,
    pub quantity: u64,
    pub taker: Address,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ListingType {
    English,
    Dutch,
}

#[derive(Debug, Clone, Copy)]
pub struct Address(ethers::abi::Address);

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct Inner {
            address: ethers::abi::Address,
        }

        Inner { address: self.0 }.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Inner {
            address: ethers::abi::Address,
        }

        Deserialize::deserialize(deserializer).map(|v: Inner| Address(v.address))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub hash: H256,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaymentToken {
    pub address: ethers::abi::Address,
    pub decimals: u64,
    pub eth_price: f64,
    pub name: String,
    pub symbol: String,
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
