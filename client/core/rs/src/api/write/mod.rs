mod action;
mod alerter;
mod api_key;
mod build;
mod builder;
mod deployment;
mod description;
mod permissions;
mod procedure;
mod provider;
mod repo;
mod server;
mod stack;
mod sync;
mod tags;
mod user;
mod user_group;
mod variable;

pub use action::*;
pub use alerter::*;
pub use api_key::*;
pub use build::*;
pub use builder::*;
pub use deployment::*;
pub use description::*;
pub use permissions::*;
pub use procedure::*;
pub use provider::*;
pub use repo::*;
pub use server::*;
pub use stack::*;
pub use sync::*;
pub use tags::*;
pub use user::*;
pub use user_group::*;
pub use variable::*;

pub trait KomodoWriteRequest: resolver_api::HasResponse {}
