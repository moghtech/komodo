//! # Komodo Core API
//!
//! Komodo Core exposes an HTTP api using standard JSON serialization.
//!
//! All calls share some common HTTP params:
//! - Method: `POST`
//! - Path: `/auth`, `/user`, `/read`, `/write`, `/execute`
//! - Headers:
//!   - Content-Type: `application/json`
//!   - Authorization: `your_jwt`
//!   - X-Api-Key: `your_api_key`
//!   - X-Api-Secret: `your_api_secret`
//!   - Use either Authorization *or* X-Api-Key and X-Api-Secret to authenticate requests.
//! - Body: JSON specifying the request type (`type`) and the parameters (`params`).
//!
//! You can create API keys for your user, or for a Service User with limited permissions,
//! from the Komodo UI Settings page.
//!
//! To call the api, construct JSON bodies following
//! the schemas in [read], [mod@write], [execute], and so on.
//!
//! For example, this is an example body for [read::GetDeployment]:
//! ```json
//! {
//!   "deployment": "my-deployment"
//! }
//! ```
//!
//! The request's parent module (eg. [read], [mod@write]) and name determines the http path which
//! must be used for the requests. For example, requests under [read] are made using http path `/read/{REQUEST_NAME}`.
//!
//! ## Curl Example
//!
//! Putting it all together, here is an example `curl` for [write::UpdateBuild], to update the version:
//!
//! ```text
//! curl --header "Content-Type: application/json" \
//!     --header "X-Api-Key: your_api_key" \
//!     --header "X-Api-Secret: your_api_secret" \
//!     --data '{ "id": "my-build", "config": { "version": "2.0.0" } }' \
//!     https://komodo.example.com/write/UpdateBuild
//! ```
//!
//! ## Modules
//!
//! - [auth]: Requests relating to logging in / obtaining authentication tokens.
//! - [read]: Read only requests which retrieve data from Komodo.
//! - [execute]: Run actions on Komodo resources, eg [execute::RunBuild].
//! - [mod@write]: Requests which alter data, like create / update / delete resources.
//!
//! ## Errors
//!
//! Request errors will be returned with a JSON body containing information about the error.
//! They will have the following common format:
//! ```json
//! {
//!   "error": "top level error message",
//!   "trace": [
//!     "first traceback message",
//!     "second traceback message"
//!   ]
//! }
//! ```

pub mod execute;
pub mod read;
pub mod terminal;
pub mod write;

pub mod auth {
  pub use mogh_auth_client::api::*;
}
