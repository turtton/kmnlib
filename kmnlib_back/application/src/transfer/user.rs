use error_stack::Report;
use kernel::prelude::entity::{DestructUser, User};
use kernel::KernelError;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UserDto {
    pub id: Uuid,
    pub name: String,
    pub rent_limit: i32,
    pub version: i64,
}

impl TryFrom<User> for UserDto {
    type Error = Report<KernelError>;
    fn try_from(value: User) -> Result<Self, Self::Error> {
        let DestructUser {
            id,
            name,
            rent_limit,
            version,
        } = value.into_destruct();
        Ok(Self {
            id: id.into(),
            name: name.into(),
            rent_limit: rent_limit.into(),
            version: version.try_into()?,
        })
    }
}

pub struct GetUserDto {
    pub id: Uuid,
}

pub struct RemoveUserDto {
    pub id: Uuid,
}
