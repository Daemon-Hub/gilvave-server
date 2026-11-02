use crate::error::{CoreError, DatabaseError};

#[derive(Debug)]
pub enum FriendError {
    UserNotFound,
    SelfRequest,
    Blocked,
    AlreadyFriends,
    RequestAlreadyExists,
    RequestNotFound,
    SelfBlock,
    Database(DatabaseError),
}

impl std::fmt::Display for FriendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserNotFound => write!(f, "User not found"),
            Self::SelfRequest => write!(f, "Cannot send friend request to yourself"),
            Self::Blocked => write!(f, "Cannot perform this action due to user block"),
            Self::AlreadyFriends => write!(f, "Users are already friends"),
            Self::RequestAlreadyExists => write!(f, "Friend request already exists"),
            Self::RequestNotFound => write!(f, "Friend request not found"),
            Self::SelfBlock => write!(f, "Cannot block yourself"),
            Self::Database(e) => write!(f, "Database error: {e}"),
        }
    }
}

impl std::error::Error for FriendError {}

impl From<DatabaseError> for FriendError {
    fn from(err: DatabaseError) -> Self {
        Self::Database(err)
    }
}

impl From<tokio_postgres::Error> for FriendError {
    fn from(err: tokio_postgres::Error) -> Self {
        Self::Database(DatabaseError::from(err))
    }
}

impl From<deadpool_postgres::PoolError> for FriendError {
    fn from(err: deadpool_postgres::PoolError) -> Self {
        Self::Database(DatabaseError::from(err))
    }
}

impl From<FriendError> for CoreError {
    fn from(err: FriendError) -> Self {
        match err {
            FriendError::UserNotFound => CoreError::NotFound(err.to_string()),
            FriendError::SelfRequest | FriendError::SelfBlock => {
                CoreError::BadRequest(err.to_string())
            }
            FriendError::Blocked => CoreError::Forbidden(err.to_string()),
            FriendError::AlreadyFriends | FriendError::RequestAlreadyExists => {
                CoreError::Conflict(err.to_string())
            }
            FriendError::RequestNotFound => CoreError::NotFound(err.to_string()),
            FriendError::Database(e) => CoreError::from(e),
        }
    }
}
