use mv64e_mtb_dto::Mtb;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::error::KafkaResult;
use rdkafka::message::{BorrowedHeaders, BorrowedMessage, Headers};
use rdkafka::Message as KafkaMessage;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub(crate) enum MessageError {
    KeyExtraction,
    RequestIdExtraction,
    PayloadExtraction {
        key: String,
        request_id: String,
    },
    PayloadDeserialization {
        key: String,
        request_id: String,
        msg: String,
    },
}

impl Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageError::KeyExtraction => write!(f, "Error extracting key"),
            MessageError::RequestIdExtraction => write!(f, "Error extracting request ID"),
            MessageError::PayloadExtraction { request_id, .. } => {
                write!(f, "Error extracting payload for '{request_id}'")
            }
            MessageError::PayloadDeserialization {
                request_id, msg, ..
            } => {
                write!(f, "Error deserializing payload for '{request_id}': {msg}")
            }
        }
    }
}

impl Error for MessageError {}

pub(crate) struct Message<'a> {
    msg: BorrowedMessage<'a>,
    key: String,
    request_id: String,
    payload: Arc<Mtb>,
}

impl<'a> TryFrom<BorrowedMessage<'a>> for Message<'a> {
    type Error = MessageError;

    fn try_from(msg: BorrowedMessage<'a>) -> Result<Message<'a>, MessageError> {
        let key = if let Some(Ok(key)) = msg.key_view::<str>() {
            key.to_string()
        } else {
            return Err(MessageError::KeyExtraction);
        };

        let request_id = if let Some(headers) = msg.headers()
            && let Some(value) = headers.iter().find(|header| header.key == "requestId")
            && let Some(value) = value.value
            && let Ok(value) = str::from_utf8(value)
        {
            value.to_string()
        } else {
            return Err(MessageError::RequestIdExtraction);
        };

        let Some(Ok(payload)) = msg.payload_view::<str>() else {
            return Err(MessageError::PayloadExtraction { key, request_id });
        };

        let payload = match serde_json::from_str::<Mtb>(payload) {
            Ok(payload) => payload,
            Err(err) => {
                return Err(MessageError::PayloadDeserialization {
                    key,
                    request_id,
                    msg: err.to_string(),
                });
            }
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
