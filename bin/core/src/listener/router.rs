use axum::{extract::Path, http::HeaderMap, routing::post, Router};
use komodo_client::entities::{
  action::Action, build::Build, procedure::Procedure, repo::Repo,
  resource::Resource, stack::Stack, sync::ResourceSync,
};
use reqwest::StatusCode;
use serde::Deserialize;
use serror::AddStatusCode;
use tracing::Instrument;

use crate::resource::KomodoResource;

use super::{
  resources::{
    handle_action_webhook, handle_build_webhook,
    handle_procedure_webhook, handle_repo_webhook,
    handle_stack_webhook, handle_sync_webhook, RepoWebhookOption,
    StackWebhookOption, SyncWebhookOption,
  },
  CustomSecret, VerifyBranch, VerifySecret,
};

#[derive(Deserialize)]
struct Id {
  id: String,
}

#[derive(Deserialize)]
struct IdAndOption<T> {
  id: String,
  option: T,
}

#[derive(Deserialize)]
struct IdAndBranch {
  id: String,
  #[serde(default = "default_branch")]
  branch: String,
}

fn default_branch() -> String {
  String::from("main")
}

pub fn router<P: VerifySecret + VerifyBranch>() -> Router {
  Router::new()
  .route(
    "/build/{id}",
    post(
      |Path(Id { id }), headers: HeaderMap, body: String| async move {
        let build =
          auth_webhook::<P, Build>(&id, headers, &body).await?;
        tokio::spawn(async move {
          let span = info_span!("BuildWebhook", id);
          async {
            let res = handle_build_webhook::<P>(
              build, body,
            )
            .await;
            if let Err(e) = res {
              warn!(
                "Failed at running webhook for build {id} | {e:#}"
              );
            }
          }
          .instrument(span)
          .await
        });
        serror::Result::Ok(())
      },
    ),
  )
  .route(
    "/repo/{id}/{option}",
    post(
      |Path(IdAndOption::<RepoWebhookOption> { id, option }), headers: HeaderMap, body: String| async move {
        let repo =
          auth_webhook::<P, Repo>(&id, headers, &body).await?;
        tokio::spawn(async move {
          let span = info_span!("RepoWebhook", id);
          async {
            let res = handle_repo_webhook::<P>(
              option, repo, body,
            )
            .await;
            if let Err(e) = res {
              warn!(
                "Failed at running webhook for repo {id} | {e:#}"
              );
            }
          }
          .instrument(span)
          .await
        });
        serror::Result::Ok(())
      },
    ),
  )
  .route(
    "/stack/{id}/{option}",
    post(
      |Path(IdAndOption::<StackWebhookOption> { id, option }), headers: HeaderMap, body: String| async move {
        let stack =
          auth_webhook::<P, Stack>(&id, headers, &body).await?;
        tokio::spawn(async move {
          let span = info_span!("StackWebhook", id);
          async {
            let res = handle_stack_webhook::<P>(
              option, stack, body,
            )
            .await;
            if let Err(e) = res {
              warn!(
                "Failed at running webhook for stack {id} | {e:#}"
              );
            }
          }
          .instrument(span)
          .await
        });
        serror::Result::Ok(())
      },
    ),
  )
  .route(
    "/sync/{id}/{option}",
    post(
      |Path(IdAndOption::<SyncWebhookOption> { id, option }), headers: HeaderMap, body: String| async move {
        let sync =
          auth_webhook::<P, ResourceSync>(&id, headers, &body).await?;
        tokio::spawn(async move {
          let span = info_span!("ResourceSyncWebhook", id);
          async {
            let res = handle_sync_webhook::<P>(
              option, sync, body,
            )
            .await;
            if let Err(e) = res {
              warn!(
                "Failed at running webhook for resource sync {id} | {e:#}"
              );
            }
          }
          .instrument(span)
          .await
        });
        serror::Result::Ok(())
      },
    ),
  )
  .route(
    "/procedure/{id}/{branch}",
    post(
      |Path(IdAndBranch { id, branch }), headers: HeaderMap, body: String| async move {
        let procedure =
          auth_webhook::<P, Procedure>(&id, headers, &body).await?;
        tokio::spawn(async move {
          let span = info_span!("ProcedureWebhook", id);
          async {
            let res = handle_procedure_webhook::<P>(
              procedure, &branch, body,
            )
            .await;
            if let Err(e) = res {
              warn!(
                "Failed at running webhook for procedure {id} | target branch: {branch} | {e:#}"
              );
            }
          }
          .instrument(span)
          .await
        });
        serror::Result::Ok(())
      },
    ),
  )
  .route(
    "/action/{id}/{branch}",
    post(
      |Path(IdAndBranch { id, branch }), headers: HeaderMap, body: String| async move {
        let action =
          auth_webhook::<P, Action>(&id, headers, &body).await?;
        tokio::spawn(async move {
          let span = info_span!("ActionWebhook", id);
          async {
            let res = handle_action_webhook::<P>(
              action, &branch, body,
            )
            .await;
            if let Err(e) = res {
              warn!(
                "Failed at running webhook for action {id} | target branch: {branch} | {e:#}"
              );
            }
          }
          .instrument(span)
          .await
        });
        serror::Result::Ok(())
      },
    ),
  )
}

async fn auth_webhook<P, R>(
  id: &str,
  headers: HeaderMap,
  body: &str,
) -> serror::Result<Resource<R::Config, R::Info>>
where
  P: VerifySecret,
  R: KomodoResource + CustomSecret,
{
  let resource = crate::resource::get::<R>(id)
    .await
    .status_code(StatusCode::BAD_REQUEST)?;
  P::verify_secret(headers, body, R::custom_secret(&resource))
    .status_code(StatusCode::UNAUTHORIZED)?;
  Ok(resource)
}
