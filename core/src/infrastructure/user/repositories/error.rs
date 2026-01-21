use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum UserError {
    #[error("User not found")]
    UserNotFound,
}

impl UserError {
    pub fn error_code(&self) -> &'static str {
        match self {
            UserError::UserNotFound => "E_USER_NOT_FOUND",
        }
    }
}
