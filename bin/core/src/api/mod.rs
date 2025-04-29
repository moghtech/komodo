pub mod auth;
pub mod execute;
pub mod read;
pub mod user;
pub mod write;

#[derive(serde::Deserialize)]
struct Variant {
  variant: String,
}
