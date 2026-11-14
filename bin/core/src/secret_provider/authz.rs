use std::{collections::BTreeSet, future::Future, str::FromStr};

use anyhow::{Context, bail};
use database::mungos::{
  find::find_collect,
  mongodb::bson::{Document, doc, oid::ObjectId},
};
use komodo_client::entities::{
  ResourceTarget, ResourceTargetVariant,
  action::Action,
  build::Build,
  deployment::Deployment,
  permission::PermissionLevel,
  procedure::Procedure,
  secret_provider::{
    ExternalSecretRef, InfisicalSecretSelector,
    InfisicalSelectorScope, SecretExecutionActor, SecretGrantSubject,
    SecretProvider, SecretProviderConfig, SecretProviderGrant,
    SecretProviderResourceUsage, SecretResourceScope, SecretSelector,
    SecretSelectorScope, SecretSource, SecretUsage,
    normalize_secret_path, secret_path_matches_prefix,
  },
  stack::Stack,
  sync::ResourceSync,
  user::User,
};

use crate::{
  helpers::query::get_user_user_group_ids,
  permission::get_check_permissions, resource, state::db_client,
};

/// Explicit execution principal propagated across direct HTTP calls, spawned tasks,
/// Procedures, Webhooks, Schedules, AutoRedeploy, ResourceSync, and Action API calls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionSubjectContext {
  /// Real human, OIDC, or admin-created Service User (never an inbuilt virtual user).
  DirectUser { user_id: String },
  /// Automated/indirect trigger with explicit actor kind and optional originating resource ID.
  SystemActor {
    actor: SecretExecutionActor,
    actor_resource_id: String,
  },
}

tokio::task_local! {
  static EXECUTION_SUBJECT: ExecutionSubjectContext;
}

pub async fn with_execution_subject<F, T>(
  subject: ExecutionSubjectContext,
  fut: F,
) -> T
where
  F: Future<Output = T>,
{
  EXECUTION_SUBJECT.scope(subject, fut).await
}

pub async fn maybe_with_execution_subject<F, T>(
  subject: Option<ExecutionSubjectContext>,
  fut: F,
) -> T
where
  F: Future<Output = T>,
{
  match subject {
    Some(s) => EXECUTION_SUBJECT.scope(s, fut).await,
    None => fut.await,
  }
}

pub fn current_execution_subject() -> Option<ExecutionSubjectContext>
{
  EXECUTION_SUBJECT.try_with(|s| s.clone()).ok()
}

/// Require a real enabled administrator (human or admin-created Service User),
/// explicitly rejecting Komodo's inbuilt virtual users (`action_user`, `procedure_user`, etc.)
/// which set `admin: true` internally.
pub fn require_real_admin_user(user: &User) -> anyhow::Result<()> {
  if !user.enabled {
    bail!("User account is disabled");
  }
  if !user.admin || User::is_service_user(&user.id) {
    bail!(
      "Admin privileges are required to manage external secret providers"
    );
  }
  Ok(())
}

/// Resolve the effective subject for external secret access.
/// Fails closed if `fallback_user` is an inbuilt virtual user without an explicit `ExecutionSubjectContext`.
pub fn resolve_effective_subject(
  fallback_user: &User,
) -> anyhow::Result<ExecutionSubjectContext> {
  if let Some(ctx) = current_execution_subject() {
    return Ok(ctx);
  }
  if fallback_user.id.trim().is_empty()
    || User::is_service_user(&fallback_user.id)
  {
    bail!(
      "External secret access denied: missing explicit execution subject context for internal actor '{}'",
      fallback_user.username
    );
  }
  Ok(ExecutionSubjectContext::DirectUser {
    user_id: fallback_user.id.clone(),
  })
}

/// Pure, deterministic evaluation of whether a single `SecretProviderGrant` authorizes
/// `(subject, user_group_ids, target, selector, usage)` for `provider_id`.
pub fn grant_matches_request(
  grant: &SecretProviderGrant,
  provider_id: &str,
  subject: &ExecutionSubjectContext,
  user_group_ids: &[String],
  target: &ResourceTarget,
  selector: &SecretSelector,
  usage: SecretUsage,
) -> anyhow::Result<bool> {
  if !grant.enabled {
    return Ok(false);
  }
  if grant.provider_id != provider_id {
    return Ok(false);
  }
  if !grant.allowed_usages.contains(&usage) {
    return Ok(false);
  }
  if !resource_matches_scopes(target, &grant.allowed_resources) {
    return Ok(false);
  }
  if !subject_matches_grant(subject, user_group_ids, &grant.subject) {
    return Ok(false);
  }
  selector_matches_scope(selector, &grant.selector_scope)
}

pub fn resource_matches_scopes(
  target: &ResourceTarget,
  scopes: &[SecretResourceScope],
) -> bool {
  scopes.iter().any(|scope| match (target, scope) {
    (
      ResourceTarget::Stack(id),
      SecretResourceScope::Stack { id: s },
    ) => !id.is_empty() && id == s,
    (ResourceTarget::Stack(_), SecretResourceScope::AllStacks) => {
      true
    }
    (
      ResourceTarget::Deployment(id),
      SecretResourceScope::Deployment { id: s },
    ) => !id.is_empty() && id == s,
    (
      ResourceTarget::Deployment(_),
      SecretResourceScope::AllDeployments,
    ) => true,
    (
      ResourceTarget::Build(id),
      SecretResourceScope::Build { id: s },
    ) => !id.is_empty() && id == s,
    (ResourceTarget::Build(_), SecretResourceScope::AllBuilds) => {
      true
    }
    _ => false,
  })
}

pub fn subject_matches_grant(
  subject: &ExecutionSubjectContext,
  user_group_ids: &[String],
  grant_subject: &SecretGrantSubject,
) -> bool {
  match (subject, grant_subject) {
    (
      ExecutionSubjectContext::DirectUser { user_id },
      SecretGrantSubject::User {
        user_id: granted_id,
      },
    ) => !user_id.is_empty() && user_id == granted_id,
    (
      ExecutionSubjectContext::DirectUser { .. },
      SecretGrantSubject::UserGroup { group_id },
    ) => {
      !group_id.is_empty()
        && user_group_ids.iter().any(|gid| gid == group_id)
    }
    (
      ExecutionSubjectContext::DirectUser { user_id },
      SecretGrantSubject::ResourceAuthorized,
    ) => !user_id.is_empty() && !User::is_service_user(user_id),
    (
      ExecutionSubjectContext::SystemActor {
        actor,
        actor_resource_id,
      },
      SecretGrantSubject::SystemActor {
        actor: granted_actor,
        actor_resource_id: granted_res_id,
      },
    ) => {
      actor == granted_actor
        && (granted_res_id.trim().is_empty()
          || (!actor_resource_id.trim().is_empty()
            && actor_resource_id.trim() == granted_res_id.trim()))
    }
    _ => false,
  }
}

pub fn selector_matches_scope(
  selector: &SecretSelector,
  scope: &SecretSelectorScope,
) -> anyhow::Result<bool> {
  match (selector, scope) {
    (
      SecretSelector::Infisical(sel),
      SecretSelectorScope::Infisical(allowed),
    ) => infisical_selector_matches_scope(sel, allowed),
  }
}

fn infisical_selector_matches_scope(
  sel: &InfisicalSecretSelector,
  allowed: &InfisicalSelectorScope,
) -> anyhow::Result<bool> {
  let project_id = sel.project_id.trim();
  let env = sel.environment.trim();
  if project_id.is_empty() || env.is_empty() {
    bail!(
      "Infisical secret selector must specify non-empty project_id and environment"
    );
  }
  let canonical_path = normalize_secret_path(&sel.secret_path)?;

  if sel.include_imports && !allowed.allow_imports {
    return Ok(false);
  }
  if sel.expand_references && !allowed.allow_expand_references {
    return Ok(false);
  }
  if !allowed.project_ids.iter().any(|p| p.trim() == project_id) {
    return Ok(false);
  }
  if !allowed.environments.iter().any(|e| e.trim() == env) {
    return Ok(false);
  }
  if allowed.path_prefixes.is_empty() {
    return Ok(false);
  }
  for prefix in &allowed.path_prefixes {
    if secret_path_matches_prefix(&canonical_path, prefix)? {
      return Ok(true);
    }
  }
  Ok(false)
}

/// Find a `SecretProvider` in MongoDB by ObjectId or immutable slug.
pub async fn find_secret_provider(
  id_or_slug: &str,
) -> anyhow::Result<SecretProvider> {
  let trimmed = id_or_slug.trim();
  if trimmed.is_empty() {
    bail!("Secret provider identifier cannot be empty");
  }
  let query = if let Ok(oid) = ObjectId::from_str(trimmed) {
    doc! { "$or": [{ "_id": oid }, { "slug": trimmed }] }
  } else {
    doc! { "slug": trimmed }
  };
  db_client()
    .secret_providers
    .find_one(query)
    .await
    .context("Failed to query database for SecretProvider")?
    .with_context(|| {
      format!("Unknown external secret provider '{trimmed}'")
    })
}

/// Resolve the effective `SecretSelector` for `(provider, resource_sources, explicit_selector)`.
pub fn resolve_selector_for_provider(
  provider: &SecretProvider,
  resource_sources: &[SecretSource],
  source_or_slug: &str,
  explicit_override: Option<&SecretSelector>,
) -> anyhow::Result<SecretSelector> {
  if let Some(sel) = explicit_override {
    validate_selector_for_provider(provider, sel)?;
    return Ok(sel.clone());
  }
  // 1. Exact match on SecretSource.name
  if let Some(src) = resource_sources.iter().find(|s| {
    !s.name.trim().is_empty()
      && s.name.trim() == source_or_slug.trim()
  }) {
    if src.provider.trim() != provider.slug
      && src.provider.trim() != provider.id
    {
      bail!(
        "SecretSource '{}' points to provider '{}', expected '{}'",
        src.name,
        src.provider,
        provider.slug
      );
    }
    validate_selector_for_provider(provider, &src.selector)?;
    return Ok(src.selector.clone());
  }
  // 2. Unnamed SecretSource matching provider slug/id
  if let Some(src) = resource_sources.iter().find(|s| {
    s.name.trim().is_empty()
      && (s.provider.trim() == provider.slug
        || s.provider.trim() == provider.id)
  }) {
    validate_selector_for_provider(provider, &src.selector)?;
    return Ok(src.selector.clone());
  }
  // 3. Provider-level default_selector (used by zero-ceremony inline [[vault.<slug>.<key>]])
  match &provider.config {
    SecretProviderConfig::Infisical(cfg) => {
      if let Some(default_sel) = &cfg.default_selector {
        let sel = SecretSelector::Infisical(default_sel.clone());
        validate_selector_for_provider(provider, &sel)?;
        return Ok(sel);
      }
    }
  }
  bail!(
    "Secret provider '{}' has no default selector configured and resource defines no SecretSource for '{}'",
    provider.slug,
    source_or_slug
  )
}

pub fn validate_selector_for_provider(
  provider: &SecretProvider,
  selector: &SecretSelector,
) -> anyhow::Result<()> {
  match (&provider.config, selector) {
    (
      SecretProviderConfig::Infisical(_),
      SecretSelector::Infisical(sel),
    ) => {
      if sel.project_id.trim().is_empty() {
        bail!(
          "Infisical selector for provider '{}' requires non-empty project_id",
          provider.slug
        );
      }
      if sel.environment.trim().is_empty() {
        bail!(
          "Infisical selector for provider '{}' requires non-empty environment",
          provider.slug
        );
      }
      normalize_secret_path(&sel.secret_path)?;
      Ok(())
    }
  }
}

fn required_resource_permission(
  usage: SecretUsage,
) -> PermissionLevel {
  match usage {
    SecretUsage::MetadataList | SecretUsage::Preview => {
      PermissionLevel::Read
    }
    SecretUsage::StackEnv
    | SecretUsage::StackComposeFileSecret
    | SecretUsage::DeploymentEnv
    | SecretUsage::BuildSecret => PermissionLevel::Execute,
  }
}

async fn verify_resource_permission_or_existence(
  target: &ResourceTarget,
  live_user: Option<&User>,
  required_level: PermissionLevel,
) -> anyhow::Result<ResourceTarget> {
  match (target, live_user) {
    (ResourceTarget::Stack(id_or_name), Some(user)) => {
      let stack = get_check_permissions::<Stack>(
        id_or_name,
        user,
        required_level.into(),
      )
      .await?;
      Ok(ResourceTarget::Stack(stack.id))
    }
    (ResourceTarget::Stack(id_or_name), None) => {
      let stack = resource::get::<Stack>(id_or_name).await?;
      Ok(ResourceTarget::Stack(stack.id))
    }
    (ResourceTarget::Deployment(id_or_name), Some(user)) => {
      let dep = get_check_permissions::<Deployment>(
        id_or_name,
        user,
        required_level.into(),
      )
      .await?;
      Ok(ResourceTarget::Deployment(dep.id))
    }
    (ResourceTarget::Deployment(id_or_name), None) => {
      let dep = resource::get::<Deployment>(id_or_name).await?;
      Ok(ResourceTarget::Deployment(dep.id))
    }
    (ResourceTarget::Build(id_or_name), Some(user)) => {
      let build = get_check_permissions::<Build>(
        id_or_name,
        user,
        required_level.into(),
      )
      .await?;
      Ok(ResourceTarget::Build(build.id))
    }
    (ResourceTarget::Build(id_or_name), None) => {
      let build = resource::get::<Build>(id_or_name).await?;
      Ok(ResourceTarget::Build(build.id))
    }
    _ => bail!(
      "External secrets are only supported on Stack, Deployment, and Build resources (got {:?})",
      target.extract_variant()
    ),
  }
}

async fn verify_system_actor_origin_exists(
  actor: SecretExecutionActor,
  actor_resource_id: &str,
) -> anyhow::Result<()> {
  let id = actor_resource_id.trim();
  if id.is_empty() {
    return Ok(());
  }
  match actor {
    SecretExecutionActor::Procedure => {
      resource::get::<Procedure>(id).await.with_context(|| {
        format!("Triggering Procedure '{id}' no longer exists")
      })?;
    }
    SecretExecutionActor::ResourceSync => {
      resource::get::<ResourceSync>(id).await.with_context(|| {
        format!("Triggering ResourceSync '{id}' no longer exists")
      })?;
    }
    SecretExecutionActor::Action => {
      resource::get::<Action>(id).await.with_context(|| {
        format!("Triggering Action '{id}' no longer exists")
      })?;
    }
    SecretExecutionActor::Webhook
    | SecretExecutionActor::Schedule
    | SecretExecutionActor::AutoRedeploy => {}
  }
  Ok(())
}

/// Live server-side authorization preflight for metadata listing, preview, config validation, and runtime execution.
/// Always re-loads User, SecretProvider, and SecretProviderGrants from MongoDB so disabled/revoked principals
/// or providers fail closed before any Infisical network call is made.
pub async fn authorize_secret_access(
  fallback_user: &User,
  provider_id_or_slug: &str,
  target: &ResourceTarget,
  selector: &SecretSelector,
  usage: SecretUsage,
  override_resource_level: Option<PermissionLevel>,
) -> anyhow::Result<SecretProvider> {
  let subject = resolve_effective_subject(fallback_user)?;
  let provider = find_secret_provider(provider_id_or_slug).await?;

  if !provider.enabled {
    bail!(
      "External secret provider '{}' ({}) is disabled",
      provider.name,
      provider.slug
    );
  }
  if !provider.has_credentials {
    bail!(
      "External secret provider '{}' ({}) has no stored credentials",
      provider.name,
      provider.slug
    );
  }

  validate_selector_for_provider(&provider, selector)?;

  let required_level = override_resource_permission(
    override_resource_level,
    required_resource_permission(usage),
  );

  let (canonical_target, user_group_ids) = match &subject {
    ExecutionSubjectContext::DirectUser { user_id } => {
      let oid = ObjectId::from_str(user_id).with_context(|| {
        format!("Invalid user_id '{user_id}' in execution context")
      })?;
      let live_user = db_client()
        .users
        .find_one(doc! { "_id": oid })
        .await
        .context("Failed to re-verify user in database")?
        .with_context(|| {
          format!("User '{user_id}' no longer exists")
        })?;
      if !live_user.enabled {
        bail!(
          "User '{}' is disabled; refusing external secret access",
          live_user.username
        );
      }
      let canonical_target = verify_resource_permission_or_existence(
        target,
        Some(&live_user),
        required_level,
      )
      .await?;
      let group_ids = get_user_user_group_ids(&live_user.id).await?;
      (canonical_target, group_ids)
    }
    ExecutionSubjectContext::SystemActor {
      actor,
      actor_resource_id,
    } => {
      verify_system_actor_origin_exists(*actor, actor_resource_id)
        .await?;
      let canonical_target = verify_resource_permission_or_existence(
        target,
        None,
        required_level,
      )
      .await?;
      (canonical_target, Vec::new())
    }
  };

  let grants = find_collect(
    &db_client().secret_provider_grants,
    doc! { "provider_id": &provider.id, "enabled": true },
    None,
  )
  .await
  .context("Failed to query SecretProviderGrants")?;

  for grant in &grants {
    if grant_matches_request(
      grant,
      &provider.id,
      &subject,
      &user_group_ids,
      &canonical_target,
      selector,
      usage,
    )? {
      return Ok(provider);
    }
  }

  bail!(
    "Access denied: no active SecretProviderGrant authorizes {:?} for provider '{}' on {:?} with usage '{}'",
    subject,
    provider.slug,
    canonical_target,
    usage
  )
}

fn override_resource_permission(
  override_level: Option<PermissionLevel>,
  default_level: PermissionLevel,
) -> PermissionLevel {
  override_level.unwrap_or(default_level)
}

/// Scan a text block for `[[vault.<slug>.<key>]]` tokens matching `provider_slug` (or all vault tokens).
pub fn collect_inline_vault_refs_in_str(
  input: &str,
) -> anyhow::Result<Vec<ExternalSecretRef>> {
  let mut refs = Vec::new();
  let mut cursor = 0usize;
  let bytes = input.as_bytes();
  while cursor + 4 <= bytes.len() {
    let Some(rel_open) = input[cursor..].find("[[") else {
      break;
    };
    let open_idx = cursor + rel_open;
    let after_open = open_idx + 2;
    let Some(rel_close) = input[after_open..].find("]]") else {
      break;
    };
    let close_idx = after_open + rel_close;
    let inner = &input[after_open..close_idx];
    if !inner.contains('\n') && !inner.contains("[[") {
      if let Some(ext_ref) =
        ExternalSecretRef::parse_bracket_token(inner)?
      {
        refs.push(ext_ref);
      }
      cursor = close_idx + 2;
    } else {
      cursor = after_open;
    }
  }
  Ok(refs)
}

/// Scan all Stacks, Deployments, and Builds in MongoDB to find every resource referencing `provider`.
pub async fn collect_secret_provider_usages(
  provider: &SecretProvider,
) -> anyhow::Result<Vec<SecretProviderResourceUsage>> {
  let db = db_client();
  let (stacks, deployments, builds) = tokio::try_join!(
    find_collect(&db.stacks, Document::new(), None),
    find_collect(&db.deployments, Document::new(), None),
    find_collect(&db.builds, Document::new(), None),
  )
  .context("Failed to scan resources for secret provider usage")?;

  let mut usages = Vec::new();

  for stack in stacks {
    let mut kinds = BTreeSet::new();
    let mut keys = BTreeSet::new();
    let matching_source_names: BTreeSet<String> = stack
      .config
      .secret_sources
      .iter()
      .filter(|s| {
        s.provider.trim() == provider.slug
          || s.provider.trim() == provider.id
      })
      .map(|s| {
        kinds.insert("secret_source".to_string());
        if s.name.trim().is_empty() {
          provider.slug.clone()
        } else {
          s.name.trim().to_string()
        }
      })
      .collect();

    if let Ok(inline_refs) =
      collect_inline_vault_refs_in_str(&stack.config.environment)
    {
      for r in inline_refs {
        if r.provider_slug == provider.slug {
          kinds.insert("inline_environment".to_string());
          keys.insert(r.secret_key);
        }
      }
    }
    for b in &stack.config.secret_bindings {
      if b.source.trim() == provider.slug
        || b.source.trim() == provider.id
        || matching_source_names.contains(b.source.trim())
      {
        kinds.insert("secret_binding".to_string());
        keys.insert(b.remote_key.clone());
      }
    }
    if !kinds.is_empty() {
      usages.push(SecretProviderResourceUsage {
        resource_type: ResourceTargetVariant::Stack,
        resource_id: stack.id,
        resource_name: stack.name,
        reference_kinds: kinds.into_iter().collect(),
        keys: keys.into_iter().collect(),
      });
    }
  }

  for dep in deployments {
    let mut kinds = BTreeSet::new();
    let mut keys = BTreeSet::new();
    let matching_source_names: BTreeSet<String> = dep
      .config
      .secret_sources
      .iter()
      .filter(|s| {
        s.provider.trim() == provider.slug
          || s.provider.trim() == provider.id
      })
      .map(|s| {
        kinds.insert("secret_source".to_string());
        if s.name.trim().is_empty() {
          provider.slug.clone()
        } else {
          s.name.trim().to_string()
        }
      })
      .collect();

    if let Ok(inline_refs) =
      collect_inline_vault_refs_in_str(&dep.config.environment)
    {
      for r in inline_refs {
        if r.provider_slug == provider.slug {
          kinds.insert("inline_environment".to_string());
          keys.insert(r.secret_key);
        }
      }
    }
    for b in &dep.config.secret_bindings {
      if b.source.trim() == provider.slug
        || b.source.trim() == provider.id
        || matching_source_names.contains(b.source.trim())
      {
        kinds.insert("secret_binding".to_string());
        keys.insert(b.remote_key.clone());
      }
    }
    if !kinds.is_empty() {
      usages.push(SecretProviderResourceUsage {
        resource_type: ResourceTargetVariant::Deployment,
        resource_id: dep.id,
        resource_name: dep.name,
        reference_kinds: kinds.into_iter().collect(),
        keys: keys.into_iter().collect(),
      });
    }
  }

  for build in builds {
    let mut kinds = BTreeSet::new();
    let mut keys = BTreeSet::new();
    let matching_source_names: BTreeSet<String> = build
      .config
      .secret_sources
      .iter()
      .filter(|s| {
        s.provider.trim() == provider.slug
          || s.provider.trim() == provider.id
      })
      .map(|s| {
        kinds.insert("secret_source".to_string());
        if s.name.trim().is_empty() {
          provider.slug.clone()
        } else {
          s.name.trim().to_string()
        }
      })
      .collect();

    if let Ok(inline_refs) =
      collect_inline_vault_refs_in_str(&build.config.secret_args)
    {
      for r in inline_refs {
        if r.provider_slug == provider.slug {
          kinds.insert("inline_secret_args".to_string());
          keys.insert(r.secret_key);
        }
      }
    }
    for b in &build.config.secret_bindings {
      if b.source.trim() == provider.slug
        || b.source.trim() == provider.id
        || matching_source_names.contains(b.source.trim())
      {
        kinds.insert("secret_binding".to_string());
        keys.insert(b.remote_key.clone());
      }
    }
    if !kinds.is_empty() {
      usages.push(SecretProviderResourceUsage {
        resource_type: ResourceTargetVariant::Build,
        resource_id: build.id,
        resource_name: build.name,
        reference_kinds: kinds.into_iter().collect(),
        keys: keys.into_iter().collect(),
      });
    }
  }

  Ok(usages)
}

#[cfg(test)]
mod tests {
  use indexmap::IndexSet;
  use komodo_client::entities::user::{
    action_user, git_webhook_user, procedure_user, sync_user,
  };

  use super::*;

  fn base_grant() -> SecretProviderGrant {
    let mut usages = IndexSet::new();
    usages.insert(SecretUsage::StackEnv);
    usages.insert(SecretUsage::MetadataList);
    SecretProviderGrant {
      id: "grant-1".into(),
      provider_id: "prov-1".into(),
      name: "api-prod".into(),
      description: String::new(),
      enabled: true,
      revision: 1,
      subject: SecretGrantSubject::ResourceAuthorized,
      allowed_resources: vec![SecretResourceScope::Stack {
        id: "stack-1".into(),
      }],
      selector_scope: SecretSelectorScope::Infisical(
        InfisicalSelectorScope {
          project_ids: vec!["proj-1".into()],
          environments: vec!["prod".into()],
          path_prefixes: vec!["/api".into()],
          allow_imports: false,
          allow_expand_references: false,
        },
      ),
      allowed_usages: usages,
      updated_at: 0,
    }
  }

  fn sel(path: &str, imports: bool, expand: bool) -> SecretSelector {
    SecretSelector::Infisical(InfisicalSecretSelector {
      project_id: "proj-1".into(),
      environment: "prod".into(),
      secret_path: path.into(),
      include_imports: imports,
      expand_references: expand,
    })
  }

  #[test]
  fn virtual_inbuilt_users_fail_closed_without_explicit_context_and_cannot_admin()
   {
    for u in [
      procedure_user(),
      git_webhook_user(),
      action_user(),
      sync_user(),
    ] {
      assert!(u.admin, "Komodo virtual users have admin=true");
      assert!(require_real_admin_user(u).is_err());
      assert!(resolve_effective_subject(u).is_err());
    }

    let real_admin = User {
      id: "670000000000000000000099".into(),
      username: "alice".into(),
      enabled: true,
      admin: true,
      ..Default::default()
    };
    assert!(require_real_admin_user(&real_admin).is_ok());
    assert_eq!(
      resolve_effective_subject(&real_admin).unwrap(),
      ExecutionSubjectContext::DirectUser {
        user_id: "670000000000000000000099".into()
      }
    );
  }

  #[test]
  fn grant_matrix_enforces_subject_resource_path_segment_imports_and_usage()
   {
    let grant = base_grant();
    let user_subj = ExecutionSubjectContext::DirectUser {
      user_id: "670000000000000000000099".into(),
    };
    let target = ResourceTarget::Stack("stack-1".into());

    // Exact path & subpath allowed
    assert!(
      grant_matches_request(
        &grant,
        "prov-1",
        &user_subj,
        &[],
        &target,
        &sel("/api", false, false),
        SecretUsage::StackEnv,
      )
      .unwrap()
    );
    assert!(
      grant_matches_request(
        &grant,
        "prov-1",
        &user_subj,
        &[],
        &target,
        &sel("/api/db", false, false),
        SecretUsage::StackEnv,
      )
      .unwrap()
    );

    // Sibling prefix "/api-other" rejected!
    assert!(
      !grant_matches_request(
        &grant,
        "prov-1",
        &user_subj,
        &[],
        &target,
        &sel("/api-other", false, false),
        SecretUsage::StackEnv,
      )
      .unwrap()
    );

    // Traversal rejected with Err!
    assert!(
      grant_matches_request(
        &grant,
        "prov-1",
        &user_subj,
        &[],
        &target,
        &sel("/api/../secret", false, false),
        SecretUsage::StackEnv,
      )
      .is_err()
    );

    // Imports & expand_references denied when grant flags are false
    assert!(
      !grant_matches_request(
        &grant,
        "prov-1",
        &user_subj,
        &[],
        &target,
        &sel("/api", true, false),
        SecretUsage::StackEnv,
      )
      .unwrap()
    );
    assert!(
      !grant_matches_request(
        &grant,
        "prov-1",
        &user_subj,
        &[],
        &target,
        &sel("/api", false, true),
        SecretUsage::StackEnv,
      )
      .unwrap()
    );

    // File secret usage denied when grant only allows StackEnv + MetadataList
    assert!(
      !grant_matches_request(
        &grant,
        "prov-1",
        &user_subj,
        &[],
        &target,
        &sel("/api", false, false),
        SecretUsage::StackComposeFileSecret,
      )
      .unwrap()
    );

    // Wrong stack denied
    assert!(
      !grant_matches_request(
        &grant,
        "prov-1",
        &user_subj,
        &[],
        &ResourceTarget::Stack("stack-2".into()),
        &sel("/api", false, false),
        SecretUsage::StackEnv,
      )
      .unwrap()
    );

    // Indirect SystemActor (Webhook/Procedure) does NOT match ResourceAuthorized grant!
    let webhook_subj = ExecutionSubjectContext::SystemActor {
      actor: SecretExecutionActor::Webhook,
      actor_resource_id: "stack-1".into(),
    };
    assert!(
      !grant_matches_request(
        &grant,
        "prov-1",
        &webhook_subj,
        &[],
        &target,
        &sel("/api", false, false),
        SecretUsage::StackEnv,
      )
      .unwrap()
    );

    // Explicit SystemActor grant with matching actor_resource_id succeeds, mismatched ID fails
    let mut proc_grant = base_grant();
    proc_grant.subject = SecretGrantSubject::SystemActor {
      actor: SecretExecutionActor::Procedure,
      actor_resource_id: "proc-allowed".into(),
    };
    assert!(
      grant_matches_request(
        &proc_grant,
        "prov-1",
        &ExecutionSubjectContext::SystemActor {
          actor: SecretExecutionActor::Procedure,
          actor_resource_id: "proc-allowed".into(),
        },
        &[],
        &target,
        &sel("/api", false, false),
        SecretUsage::StackEnv,
      )
      .unwrap()
    );
    assert!(
      !grant_matches_request(
        &proc_grant,
        "prov-1",
        &ExecutionSubjectContext::SystemActor {
          actor: SecretExecutionActor::Procedure,
          actor_resource_id: "proc-other".into(),
        },
        &[],
        &target,
        &sel("/api", false, false),
        SecretUsage::StackEnv,
      )
      .unwrap()
    );
  }
}
