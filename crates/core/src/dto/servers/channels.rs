use bytes::BytesMut;
use serde::{Deserialize, Serialize};
use tokio_postgres::types::{FromSql, IsNull, ToSql, Type};

use crate::{from_row, ids::ChannelId};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum ChannelType {
    TEXT,
    VOICE,
}

// ─── Сериализация в БД ───
impl ToSql for ChannelType {
    fn to_sql(
        &self,
        _ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
        match self {
            ChannelType::TEXT => out.extend_from_slice(b"TEXT"),
            ChannelType::VOICE => out.extend_from_slice(b"VOICE"),
        }
        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool {
        matches!(ty.kind(), tokio_postgres::types::Kind::Enum(_))
    }

    tokio_postgres::types::to_sql_checked!();
}

// ─── Десериализация из БД ───
impl<'a> FromSql<'a> for ChannelType {
    fn from_sql(
        _ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        match raw {
            b"TEXT" => Ok(ChannelType::TEXT),
            b"VOICE" => Ok(ChannelType::VOICE),
            _ => {
                let s = std::str::from_utf8(raw)?;
                Err(format!("Invalid ChannelType value: {}", s).into())
            }
        }
    }

    fn accepts(ty: &Type) -> bool {
        matches!(ty.kind(), tokio_postgres::types::Kind::Enum(_))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInfo {
    pub name: String,
    pub r#type: ChannelType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NameUpdate(pub String);

#[derive(Debug, Serialize, Deserialize)]
pub struct PositionUpdate {
    pub old: i32,
    pub new: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelView {
    pub id: ChannelId,
    pub name: String,
    pub r#type: ChannelType,
    pub position: i32,
}
from_row!(ChannelView, id, name, r#type, position);
