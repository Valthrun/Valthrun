use anyhow::anyhow;
use futures::{
    SinkExt,
    StreamExt,
};
use serde::{
    de::DeserializeOwned,
    Serialize,
};
use tokio::{
    io::{
        AsyncRead,
        AsyncWrite,
    },
    sync::mpsc,
};
use tokio_util::codec::LengthDelimitedCodec;

use crate::protocol::ClientEvent;

pub fn create_message_channel<
    R: DeserializeOwned + Send + 'static,
    T: Serialize + Send + Sync + 'static,
>(
    stream: impl AsyncRead + AsyncWrite + Send + 'static,
) -> (mpsc::Sender<T>, mpsc::Receiver<ClientEvent<R>>) {
    let (stream_rx, stream_tx) = tokio::io::split(stream);
    let (event_tx, event_rx) = mpsc::channel(16);

    tokio::spawn({
        let event_tx = event_tx.clone();
        async move {
            let mut stream = LengthDelimitedCodec::builder().new_read(stream_rx);

            while let Some(entry) = stream.next().await {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(err) => {
                        let _ = event_tx
                            .send(ClientEvent::RecvError(anyhow!("input error: {:#}", err)))
                            .await;
                        break;
                    }
                };

                let message = match bincode::deserialize::<R>(&entry) {
                    Ok(entry) => entry,
                    Err(err) => {
                        let _ = event_tx
                            .send(ClientEvent::RecvError(anyhow!(
                                "input decode error: {:#}",
                                err
                            )))
                            .await;

                        break;
                    }
                };

                let _ = event_tx.send(ClientEvent::RecvMessage(message)).await;
            }
        }
    });

    let io_tx = {
        let event_tx = event_tx.clone();
        let (tx, mut rx) = tokio::sync::mpsc::channel::<T>(16);

        tokio::spawn(async move {
            let mut stream = LengthDelimitedCodec::builder().new_write(stream_tx);
            while let Some(message) = rx.recv().await {
                let encoded = match bincode::serialize(&message) {
                    Ok(encoded) => encoded,
                    Err(err) => {
                        let _ = event_tx
                            .send(ClientEvent::SendError(anyhow!("encode error: {:#}", err)))
                            .await;
                        break;
                    }
                };

                if let Err(err) = stream.send(encoded.into()).await {
                    let _ = event_tx
                        .send(ClientEvent::SendError(anyhow!("send error: {:#}", err)))
                        .await;
                    break;
                };
            }
        });

        tx
    };

    (io_tx, event_rx)
}
