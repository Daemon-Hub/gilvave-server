use serde::{Deserialize, Serialize};
use std::fmt::Display;
use uuid::{Error, Uuid};

use tokio_postgres::types::{FromSql, ToSql, IsNull, Type};
use bytes::BytesMut;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(pub Uuid);

        impl Default for $name {
            fn default() -> Self {
                Self(Uuid::now_v7())
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
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

        // ===== TOKIO-POSTGRES IMPLEMENTATIONS =====
        impl ToSql for $name {
            fn to_sql(
                &self,
                ty: &Type,
                out: &mut BytesMut,
            ) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>>
            where
                Self: Sized,
            {
                <Uuid as ToSql>::to_sql(&self.0, ty, out)
            }

            fn accepts(ty: &Type) -> bool {
                <Uuid as ToSql>::accepts(ty)
            }

            fn to_sql_checked(
                &self,
                ty: &Type,
                out: &mut BytesMut,
            ) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
                <Uuid as ToSql>::to_sql_checked(&self.0, ty, out)
            }
        }

        impl<'a> FromSql<'a> for $name {
            fn from_sql(
                ty: &Type,
                raw: &'a [u8],
            ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
                let uuid = <Uuid as FromSql>::from_sql(ty, raw)?;
                Ok(Self(uuid))
            }

            fn accepts(ty: &Type) -> bool {
                <Uuid as FromSql>::accepts(ty)
            }
        }
    };
}

id_type!(UserId);
id_type!(MessageId);
id_type!(RefreshTokenId);