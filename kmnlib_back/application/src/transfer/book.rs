use uuid::Uuid;

use kernel::prelude::entity::{Book, DestructBook};

#[derive(Debug, Clone)]
pub struct BookDto {
    pub id: Uuid,
    pub title: String,
    pub amount: i32,
    pub version: i64,
}

impl From<Book> for BookDto {
    fn from(value: Book) -> Self {
        let DestructBook {
            id,
            title,
            amount,
            version,
        } = value.into_destruct();
        Self {
            id: id.into(),
            title: title.into(),
            amount: amount.into(),
            version: version.into(),
        }
    }
}

pub struct GetBookDto {
    pub id: Uuid,
}
