pub use crate::error::*;

mod database;
mod entity;
mod error;
mod event;
mod handler;
mod job;
mod modify;
mod query;

#[cfg(feature = "prelude")]
pub mod prelude {
    pub mod entity {
        pub use crate::entity::*;
    }
}

#[cfg(feature = "interface")]
pub mod interface {
    pub mod database {
        pub use crate::database::*;
    }
    pub mod event {
        pub use crate::event::*;
    }
    pub mod query {
        pub use crate::query::*;
    }
    pub mod update {
        pub use crate::handler::*;
        pub use crate::modify::*;
    }
    pub mod job {
        pub use crate::job::*;
    }
}
