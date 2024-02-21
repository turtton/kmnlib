use kernel::prelude::entity::{DestructUser, User};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UserDto {
    pub id: Uuid,
    pub name: String,
    pub rent_limit: i32,
    pub version: i64,
}

impl From<User> for UserDto {
    fn from(value: User) -> Self {
        let DestructUser {
            id,
            name,
            rent_limit,
            version,
        } = value.into_destruct();
        Self {
            id: id.into(),
            name: name.into(),
            rent_limit: rent_limit.into(),
            version: version.into(),
        }
    }
}

pub struct GetUserDto {
    pub id: Uuid,
}
