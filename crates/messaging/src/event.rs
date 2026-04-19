use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Event<T> {
    pub event_type: String,
    pub payload: T,
}