use crate::db::errors::DbError;
use crate::project::ConfigError;

#[derive(Debug, thiserror::Error, specta::Type)]
#[serde(tag = "type", content = "data")]
pub enum AppError {
    #[error(transparent)]
    Io(
        #[from]
        #[serde(skip)]
        std::io::Error,
    ),
    #[error(transparent)]
    Db(
        #[from]
        #[serde(skip)]
        DbError,
    ),
    #[error(transparent)]
    Config(
        #[from]
        #[serde(skip)]
        ConfigError,
    ),
    #[error("Other: `{0}`")]
    Other(String),
}

#[derive(serde::Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
enum ErrorKind {
    Io(String),
    Db(String),
    Config(String),
    Other(String),
}

// we must manually implement serde::Serialize
impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let error_message = self.to_string();
        let error_kind = match self {
            AppError::Io(_) => ErrorKind::Io(error_message),
            AppError::Db(_) => ErrorKind::Db(error_message),
            AppError::Config(_) => ErrorKind::Config(error_message),
            AppError::Other(_) => ErrorKind::Other(error_message),
        };

        error_kind.serialize(serializer)
    }
}

impl From<String> for AppError {
    fn from(error: String) -> Self {
        AppError::Other(error)
    }
}

// Add this implementation
impl From<&str> for AppError {
    fn from(error: &str) -> Self {
        AppError::Other(error.to_string())
    }
}
