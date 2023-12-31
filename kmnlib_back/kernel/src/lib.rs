mod command;
mod database;
mod entity;
mod event;
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
    pub mod command {
        pub use crate::command::*;
    }
    pub mod event {
        pub use crate::event::*;
    }
    pub mod query {
        pub use crate::query::*;
    }
    pub mod update {
        pub use crate::modify::*;
    }
}
