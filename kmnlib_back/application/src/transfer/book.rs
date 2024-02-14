use error_stack::Report;
use uuid::Uuid;

use kernel::KernelError;
use kernel::prelude::entity::{Book, DestructBook};

#[derive(Debug, Clone)]
pub struct BookDto {
    pub id: Uuid,
    pub title: String,
    pub amount: i32,
    pub version: i64,
}

impl TryFrom<Book> for BookDto {
    type Error = Report<KernelError>;
    fn try_from(value: Book) -> Result<Self, Self::Error> {
        let DestructBook {
            id,
            title,
            amount,
            version,
        } = value.into_destruct();
        Ok(Self {
            id: id.into(),
            title: title.into(),
            amount: amount.into(),
            version: version.try_into()?,
        })
    }
}

pub struct GetBookDto {
    pub id: Uuid,
}

pub struct CreateBookDto {
    pub title: String,
    pub amount: i32,
}

pub struct UpdateBookDto {
    pub id: Uuid,
    pub title: Option<String>,
    pub amount: Option<i32>,
}

pub struct DeleteBookDto {
    pub id: Uuid,
}