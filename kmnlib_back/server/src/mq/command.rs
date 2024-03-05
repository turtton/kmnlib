use crate::handler::Handler;
use application::service::{HandleBookService, HandleUserService};
use driver::database::RedisMessageQueue;
use error_stack::ResultExt;
use kernel::interface::event::{BookEvent, UserEvent};
use kernel::interface::mq::MQConfig;
use kernel::interface::mq::{ErrorOperation, MessageQueue};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandOperation {
    Book(BookEvent),
    User(UserEvent),
}

impl CommandOperation {
    pub fn book(event: BookEvent) -> Self {
        Self::Book(event)
    }

    pub fn user(event: UserEvent) -> Self {
        Self::User(event)
    }
}

pub fn init_command_worker(
    handler: &Arc<Handler>,
) -> RedisMessageQueue<Arc<Handler>, CommandOperation> {
    let handler = handler.clone();
    let pool = handler.redis_pool().clone();
    let config = MQConfig::default();
    RedisMessageQueue::new(
        pool,
        handler,
        "command_worker",
        config,
        |handler: Arc<Handler>, data: CommandOperation| async move {
            let pgpool = handler.pgpool();
            match data {
                CommandOperation::Book(book) => pgpool
                    .handle_book_event(book)
                    .await
                    .map(|_| ())
                    .change_context_lazy(|| ErrorOperation::Delay),
                CommandOperation::User(user) => pgpool
                    .handle_user_event(user)
                    .await
                    .map(|_| ())
                    .change_context_lazy(|| ErrorOperation::Delay),
            }
        },
    )
}
