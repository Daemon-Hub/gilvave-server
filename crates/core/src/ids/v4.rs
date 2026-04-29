use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(pub Uuid);

        impl Default for $name {
            fn default() -> Self {
                Self(Uuid::new_v4())
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                let uuid = uuid::Uuid::parse_str(&value).expect("Failed to parse UUID from string");
                Self(uuid)
            }
        }
    };
}

id_type!(ServerId);
id_type!(ChannelId);
