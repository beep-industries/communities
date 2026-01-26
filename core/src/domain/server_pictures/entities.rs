use std::{fmt::Display, ops::Deref, collections::HashMap};

use reqwest::Url;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone)]
pub struct PresignedUrl(Url);

impl PresignedUrl {
    pub fn new(url: Url) -> Self {
        Self(url)
    }
}

impl Deref for PresignedUrl {
    type Target = Url;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for PresignedUrl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.0.as_str())
    }
}

impl<'de> Deserialize<'de> for PresignedUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let url_str = if s.starts_with("http://") || s.starts_with("https://") {
            s
        } else {
            format!("https://{}", s)
        };
        let url = Url::parse(&url_str).map_err(serde::de::Error::custom)?;
        Ok(PresignedUrl(url))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPictureUrls {
    pub banner: PresignedUrl,
    pub picture: PresignedUrl,
}

pub type ServerPicturesMap = HashMap<crate::domain::server::entities::ServerId, ServerPictureUrls>;
