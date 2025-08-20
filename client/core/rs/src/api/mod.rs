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
//! the schemas given in [read], [mod@write], [execute], and so on.
//!
//! For example, this is an example body for [read::GetDeployment]:
//! ```json
//! {
//!   "type": "GetDeployment",
//!   "params": {
//!     "deployment": "66113df3abe32960b87018dd"
//!   }
//! }
//! ```
//!
//! The request's parent module (eg. [read], [mod@write]) determines the http path which
//! must be used for the requests. For example, requests under [read] are made using http path `/read`.
//!
//! ## Curl Example
//!
//! Putting it all together, here is an example `curl` for [write::UpdateBuild], to update the version:
//!
//! ```text
//! curl --header "Content-Type: application/json" \
//!     --header "X-Api-Key: your_api_key" \
//!     --header "X-Api-Secret: your_api_secret" \
//!     --data '{ "type": "UpdateBuild", "params": { "id": "67076689ed600cfdd52ac637", "config": { "version": "1.15.9" } } }' \
//!     https://komodo.example.com/write
//! ```
//!
//! ## Modules
//!
//! - [auth]: Requests relating to logging in / obtaining authentication tokens.
//! - [user]: User self-management actions (manage api keys, etc.)
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

pub mod auth;
pub mod execute;
pub mod read;
pub mod terminal;
pub mod user;
pub mod write;
