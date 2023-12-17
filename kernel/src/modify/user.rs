use crate::entity::{User, UserId};
use error_stack::{Context, Report};

#[async_trait::async_trait]
pub trait UserModifier<Connection>: 'static + Sync + Send {
    type Error: Context;
    async fn create(&self, con: &mut Connection, user: User) -> Result<(), Report<Self::Error>>;
    async fn update(&self, con: &mut Connection, user: User) -> Result<(), Report<Self::Error>>;
    async fn delete(
        &self,
        con: &mut Connection,
        user_id: UserId,
    ) -> Result<(), Report<Self::Error>>;
}

pub trait DependOnUserModifier<Connection>: 'static + Sync + Send {
    type UserModifier: UserModifier<Connection>;
    fn user_modifier(&self) -> &Self::UserModifier;
}
