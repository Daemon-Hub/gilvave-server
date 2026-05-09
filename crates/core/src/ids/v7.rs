use serde::{Deserialize, Serialize};
use uuid::{Error, Uuid};

#[allow(unused_imports)]
use sqlx::encode::IsNull;
#[allow(unused_imports)]
use sqlx::postgres::PgValueRef;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(pub Uuid);

        impl Default for $name {
            fn default() -> Self {
                Self(Uuid::now_v7())
            }
        }

        impl TryFrom<String> for $name {
            type Error = Error;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                match Uuid::parse_str(&value) {
                    Ok(uuid) => Ok(Self(uuid)),
                    Err(e) => Err(e),
                }
            }
        }

        impl TryFrom<&str> for $name {
            type Error = Error;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                match Uuid::parse_str(value) {
                    Ok(uuid) => Ok(Self(uuid)),
                    Err(e) => Err(e),
                }
            }
        }

        impl AsRef<Uuid> for $name {
            fn as_ref(&self) -> &Uuid {
                &self.0
            }
        }

        impl From<Uuid> for $name {
            fn from(value: Uuid) -> Self {
                Self(value)
            }
        }

        impl sqlx::Type<sqlx::Postgres> for $name {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                <uuid::Uuid as sqlx::Type<sqlx::Postgres>>::type_info()
            }
        }

        impl<'r> sqlx::Decode<'r, sqlx::Postgres> for $name {
            fn decode(
                value: PgValueRef<'r>,
            ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let uuid = <uuid::Uuid as sqlx::Decode<'r, sqlx::Postgres>>::decode(value)?;
                Ok(Self(uuid))
            }
        }
    };
}

id_type!(UserId);
id_type!(MessageId);
id_type!(RefreshTokenId);
