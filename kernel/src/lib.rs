mod command;
mod database;
mod entity;
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
    pub mod query {
        pub use crate::query::*;
    }
}
