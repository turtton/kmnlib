use crate::entity::{User, UserId};
use crate::KernelError;

#[async_trait::async_trait]
pub trait UserModifier<Connection>: 'static + Sync + Send {
    async fn create(
        &self,
        con: &mut Connection,
        user: User,
    ) -> error_stack::Result<(), KernelError>;
    async fn update(
        &self,
        con: &mut Connection,
        user: User,
    ) -> error_stack::Result<(), KernelError>;
    async fn delete(
        &self,
        con: &mut Connection,
        user_id: UserId,
    ) -> error_stack::Result<(), KernelError>;
}

pub trait DependOnUserModifier<Connection>: 'static + Sync + Send {
    type UserModifier: UserModifier<Connection>;
    fn user_modifier(&self) -> &Self::UserModifier;
}
