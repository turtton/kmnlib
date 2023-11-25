mod command;
mod entity;
mod error;
mod query;

pub use self::error::*;

#[cfg(feature = "prelude")]
pub mod prelude {
    pub mod entity {
        pub use crate::entity::*;
    }
}

#[cfg(feature = "interface")]
pub mod interface {
    pub mod command {
        pub use crate::command::*;
    }
    pub mod query {
        pub use crate::query::*;
    }
}
