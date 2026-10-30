pub mod messages;
pub mod servers;
pub mod users;

/// Макрос создает десериализатор из Row в нужный тип с поддержкой вложенных структур.
/// Для вложенных структур так же нужно вызвать данный макрос.
/// ```rust,ignore
///pub struct Profile {
///    pub name: String,
///    pub bio: String,
///    pub avatar: String,
///}
///from_row!(Profile, name, bio, avatar);
///
///pub struct UserView {
///    pub id: UserId,
///    pub username: String,
///    pub email: String,
///    pub is_active: bool,
///    pub profile: Profile    
///}
///from_row!(
///    UserView,
///    id,
///    username,
///    email,
///    is_active,
///    profile: Profile
///);
/// ```
///
#[macro_export]
macro_rules! from_row {
    // Внутреннее правило для вложенного типа (передан $type)
    (@field $row:ident, $idx:ident, $type:ty) => {
        <$type>::from_row_at($row, $idx)?
    };

    // Внутреннее правило для обычной колонки (тип не передан)
    (@field $row:ident, $idx:ident) => {{
        let val = $row.try_get(*$idx)?;
        *$idx += 1;
        val
    }};

    // Основная точка входа
    ($struct_name:ident, $( $field:ident $(: $type:ty)? ),* $(,)?) => {
        #[allow(unused_assignments)]
        impl $struct_name {
            pub fn from_row(row: &tokio_postgres::Row) -> Result<Self, $crate::error::DatabaseError> {
                Self::from_row_at(row, &mut 0)
            }

            pub fn from_row_at(
                row: &tokio_postgres::Row,
                idx: &mut usize,
            ) -> Result<Self, $crate::error::DatabaseError> {
                Ok(Self {
                    $(
                        $field: $crate::from_row!(@field row, idx $(, $type)?),
                    )*
                })
            }
        }
    };
}
