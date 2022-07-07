#![deny(missing_docs)]

//! Crate for receiving updates from the [OpenSea Stream API](https://docs.opensea.io/reference/stream-api-overview).
//! This crate is a thin wrapper over [`phyllo`] with a few convenience functions and struct definitions for the event schema.
//! It is recommended that you also read the documentation of [`phyllo`] to understand the Phoenix protocol which delivers these messages.
//!
//! # Example
//! The following example prints all listings of items in the `wandernauts` collection as they are created.
//! ```no_run
//! # use opensea_stream::{client, schema, subscribe_to, Collection, Network};
//! # use phyllo::message::Payload;
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let mut client = client(Network::Mainnet, "YOUR_API_KEY_HERE").await;
//!
//!     // Subscribe to a collection. Note that you must all subscribe to all events
//!     // in the collection; filtering is your responsibility (see below).
//!     let (handler, mut subscription) = subscribe_to(
//!         &mut client,
//!         Collection::Collection("wandernauts".to_string()),
//!     )
//!     .await?;
//!
//!     // To unsubscribe:
//!     // handler.close().await?;
//!
//!     loop {
//!         // The message received from the channel is a raw message of the Phoenix protocol.
//!         // It may or may not contain a payload.
//!         let event = match subscription.recv().await?.into_custom_payload() {
//!             Some(v) => v,
//!             None => {
//!                 eprintln!("unexpected message");
//!                 continue;
//!             }
//!         };
//!
//!         // Only print item listing events.
//!         if let schema::Payload::ItemListed(listing) = event.payload {
//!             println!("{:?}", listing);
//!         }
//!     }
//! }
//! ```
//! # Features
//! `rustls-tls-native-roots` (which uses [`rustls-native-certs`](https://crates.io/crates/rustls-native-certs)
//! for root certificates) is enabled by default. To use `rustls-tls-webpki-roots` ([`webpki-roots`](https://crates.io/crates/webpki-roots))
//! instead, include this in your `Cargo.toml`:
//! ```toml
//! opensea-stream = { version = "0.1", default-features = false, features = ["rustls-tls-webpki-roots"] }
//! ```

use phyllo::{
    channel::{ChannelBuilder, ChannelHandler},
    error::RegisterChannelError,
    message::Message,
    socket::{SocketBuilder, SocketHandler},
};
use schema::StreamEvent;
use serde_json::Value;
use tokio::sync::broadcast;
use url::Url;

pub use phyllo;

mod protocol;
/// Payload schema for messages received from the websocket.
pub mod schema;

pub use protocol::*;

/// Create a client.
pub async fn client(network: Network, token: &str) -> SocketHandler<Collection> {
    let mut network: Url = Url::from(network);
    network.query_pairs_mut().append_pair("token", token);
    SocketBuilder::new(network).build().await
}

/// Subscribe to all the events of a particular [`Collection`].
pub async fn subscribe_to(
    socket: &mut SocketHandler<Collection>,
    collection: Collection,
) -> Result<
    (
        ChannelHandler<Collection, Event, Value, StreamEvent>,
        broadcast::Receiver<Message<Collection, Event, Value, StreamEvent>>,
    ),
    RegisterChannelError,
> {
    socket.channel(ChannelBuilder::new(collection)).await
}
