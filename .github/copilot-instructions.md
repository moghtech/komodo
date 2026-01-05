# Komodo AI Coding Instructions

## Project Overview
Komodo is a distributed system for building and deploying software across multiple servers. It consists of:
- **Core** (`bin/core`): Central Rust/Axum API server managing resources, scheduling, monitoring
- **Periphery** (`bin/periphery`): Rust agent deployed to managed servers, executes docker/git operations
- **Frontend** (`frontend`): React/TypeScript SPA using TanStack Query and shadcn/ui components
- **Client Libraries** (`client/core/{rs,ts}`): Type-safe API clients generated from shared entity definitions

## Architecture Fundamentals

### Core ↔ Periphery Communication
- Core and Periphery communicate via **WebSocket connections** that can be initiated from either direction
- Periphery can connect outbound to Core (common) or Core can connect inbound to Periphery (less common)
- Connection management is in `bin/core/src/connection` and `bin/periphery/src/connection`
- Use `state::periphery_connections()` in Core to send requests to Periphery agents

### Type Generation Pipeline
**Critical**: Rust entities are the source of truth for all types across the stack:
1. Entities defined in `client/core/rs/src/entities/` with `#[typeshare]` macro
2. Run `node ./client/core/ts/generate_types.mjs` (uses typeshare CLI) → generates `client/core/ts/src/types.ts`
3. TypeScript client is built and linked to frontend via `yarn link komodo_client`
4. Frontend imports types: `import { Types } from "komodo_client"`

**When modifying entities**: Always run the `gen-client` (alias: `gc`) runfile task to regenerate TypeScript types.

### API Pattern (Resolver-Based)
Core and Periphery use `resolver_api::Resolve` trait for RPC-style endpoints:
- Request enums in `client/*/rs/src/api/` implement `Resolve<Response = T, Error = E>`
- Core handlers in `bin/core/src/api/{read,write,execute}/*.rs`
- Periphery handlers in `bin/periphery/src/api/*.rs`
- Router maps `/{variant}` paths to enum variants automatically

Example request handling:
```rust
#[derive(Serialize, Deserialize, Resolve)]
#[response(MyResponse)]
#[error(anyhow::Error)]
pub struct MyRequest { /* fields */ }
```

### State Management
Core maintains singleton state via `OnceLock` patterns in `bin/core/src/state.rs`:
- `db_client()`: MongoDB connection
- `periphery_connections()`: WebSocket connections to Periphery agents
- `action_states()`, `build_states()`, etc.: In-memory caches using `CloneCache<K, V>`
- Always use `state::*()` accessors, never instantiate directly

## Development Workflows

### Running Services Locally
```bash
# Start all local services (MongoDB, Core, Periphery, Frontend)
run dc  # dev-core: runs Core in release mode with .dev/core.config.toml
run dp  # dev-periphery: runs Periphery with .dev/periphery.config.toml

# Or use docker-compose for full stack
run dev-compose  # deploys dev.compose.yaml with ferretdb
```

### Frontend Development
```bash
cd frontend && yarn dev  # Vite dev server on port 5173

# After changing Rust entities, regenerate client:
run gc  # gen-client: typeshare → build TS client → copy to frontend/public/client
```

### Building & Deploying
- Deno scripts in `action/` automate version bumps and deployments
- `run dk` (deploy-komodo): Full build and deploy via Komodo Action
- `run bk` (build-komodo): Build binaries without deploying

## Code Conventions

### Error Handling
- Use `anyhow::Result<T>` for application code
- Use `serror` for serializable errors sent over API boundaries
- Response helper: `response::Response::from(value)` for JSON responses
- Never use `unwrap()` in production code paths; prefer `context()` for error messages

### Naming & Structure
- **Entities**: CamelCase structs in `client/core/rs/src/entities/*/mod.rs`
- **API requests**: CamelCase request types in `client/core/rs/src/api/*.rs`
- **Config derivations**: Use `derive_builder::Builder` for config update structs
- **Variants**: `derive_variants::EnumVariants` generates `TypeVariant` enums for discriminants

### Frontend Patterns
- **API hooks**: `useRead("GetStack", { stack: "my-stack" })` for queries
- **Mutations**: `useWrite("UpdateStack", { onSuccess: () => invalidate(...) })`
- **Components**: Located in `frontend/src/components/`, use shadcn/ui primitives from `@ui/*`
- **State**: Prefer TanStack Query for server state, Jotai atoms for local UI state

## Key Files & Directories

### Must-Read for Context
- [readme.md](readme.md): Project overview and deployment info
- [runfile.toml](runfile.toml): All development task definitions
- [bin/core/src/state.rs](bin/core/src/state.rs): Core's global state singletons
- [client/core/rs/src/entities/mod.rs](client/core/rs/src/entities/mod.rs): All entity definitions

### Common Touch Points
- **Adding new resource type**: Update entities, Core API handlers, Frontend pages/components
- **Changing API**: Modify request/response types in `client/core/rs/src/api/`, regenerate client
- **Frontend features**: Usually start in `frontend/src/pages/`, add hooks in `frontend/src/lib/hooks/`
- **Periphery capabilities**: Add to `periphery_client::api` enum and implement handler in `bin/periphery/src/api/`

## Configuration
- Core config: `config/core.config.toml` (see `client/core/rs/src/entities/config/core.rs` for schema)
- Periphery config: `config/periphery.config.toml`
- Environment variable pattern: Settings support `_FILE` suffix for Docker secrets (see `lib/environment_file`)

## Testing & Validation
- No comprehensive test suite currently
- Validate changes by running locally with `run dc` / `run dp`
- Check Core logs for startup errors (connection to DB, Periphery agents)
- Frontend errors appear in browser console and TanStack Query devtools

## Common Pitfalls
- **Forgetting to regenerate TS types** after Rust entity changes → frontend type errors
- **Not using `state::*()` accessors** → panics from uninitialized `OnceLock`
- **Breaking WebSocket message format** between Core/Periphery → connection failures
- **Yarn link issues**: If frontend can't find `komodo_client`, re-run `run link-client`
