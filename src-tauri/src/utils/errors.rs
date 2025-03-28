use crate::db::DbError;

#[derive(Debug, thiserror::Error, specta::Type)]
#[serde(tag = "type", content = "data")]
pub enum Error {
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
    #[error("Other: `{0}`")]
    Other(String),
}

#[derive(serde::Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
enum ErrorKind {
    Io(String),
    Db(String),
    Other(String),
}

// we must manually implement serde::Serialize
impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let error_message = self.to_string();
        let error_kind = match self {
            Error::Io(_) => ErrorKind::Io(error_message),
            Error::Db(_) => ErrorKind::Db(error_message),
            Error::Other(_) => ErrorKind::Other(error_message),
        };

        error_kind.serialize(serializer)
    }
}

pub type CommandResult<T> = Result<T, Error>;
