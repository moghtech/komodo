# Komodo
*A system to build and deploy software across many servers*. [https://komo.do](https://komo.do)

Docs: [https://docs.rs/komodo_client/latest/komodo_client](https://docs.rs/komodo_client/latest/komodo_client).

This is a client library for the Komodo Core API.
It contains:
- Definitions for the application [api](https://docs.rs/komodo_client/latest/komodo_client/api/index.html)
	and [entities](https://docs.rs/komodo_client/latest/komodo_client/entities/index.html).
- A [client](https://docs.rs/komodo_client/latest/komodo_client/struct.KomodoClient.html)
	to interact with the Komodo Core API.
- Information on configuring Komodo
	[Core](https://docs.rs/komodo_client/latest/komodo_client/entities/config/core/index.html) and
	[Periphery](https://docs.rs/komodo_client/latest/komodo_client/entities/config/periphery/index.html).

## Client Configuration

The client includes a convenenience method to parse the Komodo API url and credentials from the environment:
- `KOMODO_ADDRESS`
- `KOMODO_API_KEY`
- `KOMODO_API_SECRET`

## Client Example
```rust
dotenvy::dotenv().ok();

let client = KomodoClient::new_from_env()?;

// Get all the deployments
let deployments = client.read(ListDeployments::default()).await?;

println!("{deployments:#?}");

let update = client.execute(RunBuild { build: "test-build".to_string() }).await?:
```