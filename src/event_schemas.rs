use chrono::{DateTime, Utc};
use ethnum::U256;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct StreamEvent {
    pub sent_at: DateTime<Utc>,
    #[serde(flatten)]
    pub payload: Payload,
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct Context {
    pub collection: Collection,
    pub item: Item,
}

#[derive(Debug)]
pub struct Collection(String);

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

#[derive(Deserialize, Debug)]
pub struct Item {
    pub nft_id: String,
    pub permalink: String,
    pub chain: Chain,
    pub metadata: Metadata,
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct Metadata {
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub animation_url: Option<String>,
    pub metadata_url: String,
}

#[derive(Deserialize, Debug)]
pub struct ItemListedData {
    #[serde(flatten)]
    pub context: Context,

    pub event_timestamp: DateTime<Utc>,
    #[serde(with = "ethnum::serde::decimal")]
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

#[derive(Deserialize, Debug)]
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
    #[serde(with = "ethnum::serde::decimal")]
    pub sale_price: U256,
    pub taker: Address,
    pub transaction: Transaction,
}

#[derive(Deserialize, Debug)]
pub struct ItemTransferredData {
    #[serde(flatten)]
    pub context: Context,

    pub event_timestamp: DateTime<Utc>,
    pub transaction: Transaction,
    pub from_account: Address,
    pub to_account: Address,
    pub quantity: u64,
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct ItemCancelledData {
    #[serde(flatten)]
    pub context: Context,

    pub event_timestamp: DateTime<Utc>,
    pub listing_type: Option<ListingType>,
    pub payment_token: PaymentToken,
    pub quantity: u64,
    pub transaction: Transaction,
}

#[derive(Deserialize, Debug)]
pub struct ItemReceivedOfferData {
    #[serde(flatten)]
    pub context: Context,

    pub event_timestamp: DateTime<Utc>,
    #[serde(with = "ethnum::serde::decimal")]
    pub base_price: U256,
    pub created_date: DateTime<Utc>,
    pub expiration_date: DateTime<Utc>,
    pub maker: Address,
    pub payment_token: PaymentToken,
    pub quantity: u64,
    pub taker: Option<Address>,
}

#[derive(Deserialize, Debug)]
pub struct ItemReceivedBidData {
    #[serde(flatten)]
    pub context: Context,

    pub event_timestamp: DateTime<Utc>,
    #[serde(with = "ethnum::serde::decimal")]
    pub base_price: U256,
    pub created_date: DateTime<Utc>,
    pub expiration_date: DateTime<Utc>,
    pub maker: Address,
    pub payment_token: PaymentToken,
    pub quantity: u64,
    pub taker: Address,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ListingType {
    English,
    Dutch,
}

#[derive(Debug)]
pub struct Address(String);

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Inner {
            address: String,
        }

        Deserialize::deserialize(deserializer).map(|v: Inner| Address(v.address))
    }
}

#[derive(Deserialize, Debug)]
pub struct Transaction {
    pub hash: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
pub struct PaymentToken {
    pub address: String,
    pub decimals: u64,
    pub eth_price: f64,
    pub name: String,
    pub symbol: String,
    pub usd_price: f64,
}
