use komodo_client::entities::config::cli::args::create::CreateCommand;

mod api_key;
mod onboarding_key;

pub async fn handle(command: &CreateCommand) -> anyhow::Result<()> {
  match command {
    CreateCommand::ApiKey(api_key) => api_key::create(api_key).await,
    CreateCommand::OnboardingKey(onboarding_key) => {
      onboarding_key::create(onboarding_key).await
    }
  }
}
