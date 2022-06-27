use collection::Collection;
use event::Event;
use event_schemas::StreamEvent;
use phyllo::{
    channel::{channel_builder::ChannelBuilder, ChannelHandler},
    message::Message,
    socket::{socket_builder::SocketBuilder, SocketHandler},
};
use serde_json::Value;
use tokio::sync::broadcast;
use url::Url;

pub mod collection;
pub mod event;
pub mod event_schemas;

pub enum Network {
    Mainnet,
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

pub async fn client(network: Network, token: &str) -> SocketHandler<Collection> {
    let mut network: Url = Url::from(network);
    network.query_pairs_mut().append_pair("token", token);
    SocketBuilder::new(network).build().await
}

pub async fn subscribe_to(
    socket: &mut SocketHandler<Collection>,
    collection: Collection,
) -> (
    ChannelHandler<Collection, Event, Value, StreamEvent>,
    broadcast::Receiver<Message<Collection, Event, Value, StreamEvent>>,
) {
    socket.channel(ChannelBuilder::new(collection)).await
}
