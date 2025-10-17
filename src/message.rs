use mv64e_mtb_dto::Mtb;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::error::KafkaResult;
use rdkafka::message::{BorrowedHeaders, BorrowedMessage, Headers};
use rdkafka::Message as KafkaMessage;
use std::sync::Arc;

pub struct Message<'a> {
    msg: BorrowedMessage<'a>,
    key: String,
    request_id: String,
    payload: Arc<Mtb>,
}

impl<'a> TryFrom<BorrowedMessage<'a>> for Message<'a> {
    type Error = String;

    fn try_from(msg: BorrowedMessage<'a>) -> Result<Message<'a>, String> {
        let key = if let Some(Ok(key)) = msg.key_view::<str>() {
            key.to_string()
        } else {
            return Err("Error getting key".into());
        };

        let request_id = match msg.headers() {
            None => None,
            Some(headers) => {
                if let Some(value) = headers.iter().find(|header| header.key == "requestId") {
                    if let Some(value) = value.value {
                        match str::from_utf8(value) {
                            Ok(value) => Some(value.to_string()),
                            Err(_) => None,
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        };

        let Some(request_id) = request_id else {
            return Err("Error getting request ID".into());
        };

        let Some(Ok(payload)) = msg.payload_view::<str>() else {
            return Err("Error getting payload".into());
        };

        let Ok(payload) = serde_json::from_str::<Mtb>(payload) else {
            return Err("Error deserializing payload".into());
        };

        Ok(Message::<'a> {
            msg,
            key,
            request_id,
            payload: Arc::new(payload),
        })
    }
}

impl Message<'_> {
    pub(crate) fn key(&self) -> String {
        self.key.clone()
    }

    pub(crate) fn request_id(&self) -> String {
        self.request_id.clone()
    }

    pub(crate) fn headers(&self) -> Option<&BorrowedHeaders> {
        self.msg.headers()
    }

    pub(crate) fn payload(&self) -> Arc<Mtb> {
        self.payload.clone()
    }

    pub(crate) fn commit(self, consumer: &StreamConsumer) -> KafkaResult<()> {
        consumer.commit_message(&self.msg, CommitMode::Async)
    }
}
