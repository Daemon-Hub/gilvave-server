pub mod channel;
pub mod message;
pub mod server;
pub mod user;

#[macro_export]
macro_rules! from_row {
    ($struct_name:ident, $($field:ident),* $(,)?) => {
        #[allow(unused_assignments)]
        impl $struct_name {
            pub fn from_row(row: &tokio_postgres::Row) -> Result<Self, $crate::error::DatabaseError> {
                let mut idx = 0usize;
                Ok($struct_name {
                    $(
                        $field: {
                            let val = row.try_get(idx)?;
                            idx += 1;
                            val
                        }
                    ),*
                })
            }
        }
    };
}
