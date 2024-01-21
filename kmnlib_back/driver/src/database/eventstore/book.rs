use error_stack::{FutureExt, Report, ResultExt};
use eventstore::Client;

use kernel::interface::command::{BookCommand, BookCommandHandler, BOOK_STREAM_NAME};
use kernel::interface::event::{BookEvent, EventInfo, UserEvent};
use kernel::interface::query::BookEventQuery;
use kernel::prelude::entity::{Book, BookId, EventVersion, User};
use kernel::KernelError;

use crate::database::eventstore::{append_event, read_stream};

pub struct EventStoreBookHandler {
    client: Client,
}

impl EventStoreBookHandler {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl BookCommandHandler for EventStoreBookHandler {
    async fn handle(
        &self,
        command: BookCommand,
    ) -> error_stack::Result<EventVersion<Book>, KernelError> {
        let (event_type, id, rev_version, event) = BookEvent::convert(command);
        append_event(
            &self.client,
            BOOK_STREAM_NAME,
            event_type,
            Some(&id.as_ref().to_string()),
            rev_version,
            event,
        )
        .await
    }
}

#[async_trait::async_trait]
impl BookEventQuery for EventStoreBookHandler {
    async fn get_events(
        &self,
        id: &BookId,
        since: Option<&EventVersion<Book>>,
    ) -> error_stack::Result<Vec<EventInfo<BookEvent, Book>>, KernelError> {
        read_stream(
            &self.client,
            BOOK_STREAM_NAME,
            Some(&id.as_ref().to_string()),
            since,
        )
        .await?
        .into_iter()
        .map(|event| {
            event
                .revision
                .try_into()
                .map_err(|e| Report::from(e).change_context(KernelError::Internal))
                .and_then(|version: i64| {
                    event
                        .as_json::<BookEvent>()
                        .map(|event| EventInfo::new(event, EventVersion::new(version)))
                        .change_context_lazy(|| KernelError::Internal)
                })
        })
        .collect::<error_stack::Result<Vec<EventInfo<BookEvent, Book>>, KernelError>>()
    }
}

#[cfg(test)]
mod test {
    use uuid::Uuid;

    use kernel::interface::command::{BookCommand, BookCommandHandler};
    use kernel::interface::event::BookEvent;
    use kernel::interface::query::BookEventQuery;
    use kernel::prelude::entity::{BookAmount, BookId, BookTitle};
    use kernel::KernelError;

    use crate::database::eventstore::{create_event_store_client, EventStoreBookHandler};

    #[test_with::env(EVENTSTORE_TEST)]
    #[tokio::test]
    async fn basic_modification() -> error_stack::Result<(), KernelError> {
        let client = create_event_store_client()?;
        let handler = EventStoreBookHandler::new(client);
        let id = BookId::new(Uuid::new_v4());

        let create_book = BookCommand::Create {
            id: id.clone(),
            title: BookTitle::new("test".to_string()),
            amount: BookAmount::new(1),
        };
        let _ = handler.handle(create_book.clone()).await?;

        let mut expected = vec![BookEvent::convert(create_book).3];
        assert_eq!(handler.get_events(&id, None).await?, expected);

        let update_book = BookCommand::Update {
            id: id.clone(),
            title: Some(BookTitle::new("test2".to_string())),
            amount: None,
        };
        let _ = handler.handle(update_book.clone()).await?;

        expected.push(BookEvent::convert(update_book).3);
        assert_eq!(handler.get_events(&id, None).await?, expected);

        let delete_book = BookCommand::Delete { id: id.clone() };
        let _ = handler.handle(delete_book.clone()).await?;

        expected.push(BookEvent::convert(delete_book).3);
        assert_eq!(handler.get_events(&id, None).await?, expected);
        Ok(())
    }
}
