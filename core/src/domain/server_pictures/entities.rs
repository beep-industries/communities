use std::fmt::Display;

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
