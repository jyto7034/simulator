# Repository Guidelines

## Project Structure & Modules
- Root is a Rust workspace (`Cargo.toml`).
- Core library: `simulator_core/` (shared types, logic, assets).
- Servers: `simulator_auth_server/`, `simulator_match_server/`, `simulator_dedicated_server/`.
- Client app: `simulator_client/` (Svelte + Tauri in `src-tauri/`).
- Metrics and config: `simulator_metrics/`, `simulator_env/`.
- Integration tests live under each crate’s `tests/`.
- Config files: root `simulator.toml`, per-service `.env` files.

## Build, Test, Run
- Build all crates: `cargo build --workspace`.
- Build single crate: `cargo build -p simulator_core`.
- Run a server: `cargo run -p simulator_match_server` (similar for `simulator_auth_server`).
- Run client (web): `cd simulator_client && npm ci && npm run dev`.
- Run client (Tauri): `cd simulator_client && npm run tauri dev`.
- Test all Rust crates: `cargo test --workspace`.
- Test specific crate: `cargo test -p simulator_core`.

## Coding Style & Naming
- Rust 2021 edition, 4-space indent; format with `cargo fmt`.
- Lint strictly: `cargo clippy --workspace -- -D warnings`.
- Naming: snake_case (functions/modules), UpperCamelCase (types), SCREAMING_SNAKE_CASE (consts).
- Client TypeScript/Svelte: keep components in `simulator_client/src/`; run `npm run check` for type/svelte checks.

## Testing Guidelines
- Prefer unit tests close to code (`src/..._tests.rs`) and integration tests in `tests/`.
- Name tests after behavior (e.g., `handles_invalid_token`).
- Use feature flags or test helpers from `simulator_core` to avoid duplication.
- Run `cargo test -p <crate>` before opening a PR.

## Commit & PR Guidelines
- Follow Conventional Commits (see `GIT_COMMIT_CONVENTION.md`).
  - Example: `feat(core): add card validation rules`.
- PRs must include: clear description, linked issues (e.g., `Closes #123`), test coverage notes, and screenshots/logs when UI or behavior changes.
- Keep changes scoped per crate when possible; cross-crate changes should note version impacts.

## Security & Configuration
- Do not commit secrets. Use `.env` files (e.g., `simulator_auth_server/.env`).
- Document required vars in PRs touching config.
- Local overrides belong in `simulator.toml` or per-crate config directories.

## Architecture Notes
- Shared domain logic resides in `simulator_core`; services depend on it.
- Servers expose APIs and orchestrate matches; the client (Svelte/Tauri) consumes them.
