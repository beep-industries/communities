use std::{collections::HashMap, fmt::Display, ops::Deref};

use reqwest::Url;
use serde::{Deserialize, Serialize};
use tracing::debug;
pub enum Content {
    ServerPicture,
    ServerBanner,
}

impl Display for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Content::ServerPicture => write!(f, "server_picture"),
            Content::ServerBanner => write!(f, "server_banner"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ContentVerb {
    Put,
    Get,
}

impl Display for ContentVerb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentVerb::Get => write!(f, "Put"),
            ContentVerb::Put => write!(f, "Get"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresignedUrl {
    url: String,
}

impl PresignedUrl {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

impl Deref for PresignedUrl {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.url
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPictureUrls {
    pub banner: PresignedUrl,
    pub picture: PresignedUrl,
}

pub type ServerPicturesMap = HashMap<crate::domain::server::entities::ServerId, ServerPictureUrls>;
