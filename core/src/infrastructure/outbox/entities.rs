use futures_util::StreamExt;
use sqlx::postgres::PgListener;

use crate::domain::outbox::{
    entities::{OutboxMessage, OutboxMessageStream},
    error::OutboxError,
};

impl From<PgListener> for OutboxMessageStream {
    fn from(value: PgListener) -> Self {
        let outbox_event_stream =
            value
                .into_stream()
                .map(|pg_notification| -> Result<OutboxMessage, OutboxError> {
                    let notification = match pg_notification {
                        Ok(notification) => notification,
                        Err(e) => {
                            return Err(OutboxError::ListenerError { msg: e.to_string() });
                        }
                    };
                    let notif = notification.payload();

                    // Parse the notification wrapper structure
                    let wrapper: serde_json::Value = serde_json::from_str(notif)
                        .map_err(|e| OutboxError::ListenerError { msg: e.to_string() })?;

                    // Extract the "data" field which contains the OutboxMessage
                    let data = wrapper.get("data").ok_or(OutboxError::ListenerError {
                        msg: "Could not find any data inside payload".to_string(),
                    })?;

                    // Deserialize the data field into OutboxMessage
                    let message: OutboxMessage = serde_json::from_value(data.clone())
                        .map_err(|e| OutboxError::ListenerError { msg: e.to_string() })?;

                    Ok(message)
                });
        OutboxMessageStream::new(outbox_event_stream)
    }
}
