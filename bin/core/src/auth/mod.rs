use std::{sync::Arc, time::Instant};

use axum::{
  extract::Request, http::HeaderMap, middleware::Next,
  response::Response, routing::post, Extension, Json, Router,
};
use axum_extra::{headers::ContentType, TypedHeader};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use resolver_api::Resolver;
use serror::{AppError, AuthError};
use uuid::Uuid;

mod github;
mod google;
mod jwt;
mod local;

use crate::{api::auth::AuthRequest, state::State};

pub use self::jwt::{
  jwt_client, InnerRequestUser, RequestUser, RequestUserExtension,
};
use self::{
  github::client::github_oauth_client,
  google::client::google_oauth_client,
};

pub async fn auth_request(
  state: Extension<Arc<State>>,
  headers: HeaderMap,
  mut req: Request,
  next: Next,
) -> Result<Response, AuthError> {
  let user = state.authenticate_check_enabled(&headers).await?;
  req.extensions_mut().insert(user);
  Ok(next.run(req).await)
}

pub fn router() -> Router {
  let mut router = Router::new().route(
    "/",
    post(|Json(request): Json<AuthRequest>| async move {
      let timer = Instant::now();
      let req_id = Uuid::new_v4();
      info!(
        "/auth request {req_id} | METHOD: {}",
        request.req_type()
      );
      let res = State.resolve_request(request, ()).await;
      if let Err(e) = &res {
        info!("/auth request {req_id} | ERROR: {e:?}");
      }
      let res = res?;
      let elapsed = timer.elapsed();
      info!("/auth request {req_id} | resolve time: {elapsed:?}");
      debug!("/auth request {req_id} | RESPONSE: {res}");
      Result::<_, AppError>::Ok((
        TypedHeader(ContentType::json()),
        res,
      ))
    }),
  );

  if github_oauth_client().is_some() {
    router = router.nest("/github", github::router())
  }

  if google_oauth_client().is_some() {
    router = router.nest("/google", google::router())
  }

  router
}

pub fn random_string(length: usize) -> String {
  thread_rng()
    .sample_iter(&Alphanumeric)
    .take(length)
    .map(char::from)
    .collect()
}

// fn random_duration(min_ms: u64, max_ms: u64) -> Duration {
//     Duration::from_millis(thread_rng().gen_range(min_ms..max_ms))
// }
