use serde::{Deserialize, Serialize};

use crate::domain::server_pictures::ContentVerb;

#[derive(Serialize, Debug, Deserialize)]
pub struct RequestSignUrl {
    action: ContentVerb,
    expires_in_ms: u32,
}

impl From<ContentVerb> for RequestSignUrl {
    fn from(value: ContentVerb) -> Self {
        Self::new(value)
    }
}

impl RequestSignUrl {
    pub fn new(action: ContentVerb) -> Self {
        Self {
            action,
            expires_in_ms: 10000,
        }
    }
}
