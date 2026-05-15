# SeaORM + SQLite Auth Plan

## Context
- The app is currently a minimal Dioxus 0.7.1 template with `router` and `fullstack` enabled in `Cargo.toml`.
- Current routes/components are demo pages (`Home`, `Blog`, `Navbar`) plus a demo server function in `src/components/echo.rs`.
- There is no existing database layer, persistence setup, auth flow, user model, or protected route logic yet.
- Goal: add an auth foundation using SeaORM with SQLite in a way that fits the current Dioxus fullstack architecture.

## Approach
- Use Dioxus server-side functions/endpoints for auth operations and keep database access on the server only.
- Use SeaORM + SQLite for persistence.
- Prefer server-managed session auth over storing sensitive auth state directly in the client.
- Add a small auth slice first: user table, password hashing, login/logout/session lookup, and a protected-page/auth-state pattern that fits Dioxus routing.
- Scope and exact implementation details still need clarification from the user.

## Files to modify
- `Cargo.toml`
- `src/main.rs`
- `src/components/echo.rs` (likely replaced or supplemented by real server functions/patterns)
- `src/views/home.rs`
- `src/views/navbar.rs`
- `src/views/mod.rs`
- likely new files/modules for database, models/entities, auth server functions, and auth UI

## Reuse
- Reuse Dioxus fullstack server function pattern from `src/components/echo.rs`
- Reuse routing/layout structure from `src/main.rs` and `src/views/navbar.rs`
- Reuse current Dioxus 0.7 app/module organization (`components`, `views`)

## Steps
- [ ] Clarify target platforms (web/desktop/mobile), auth scope, and whether registration is needed
- [ ] Add SeaORM + SQLite dependencies and server-only database initialization path
- [ ] Define initial schema/entities for users (and likely sessions)
- [ ] Add password hashing and credential verification on the server
- [ ] Implement login/logout/current-user server functions or endpoints
- [ ] Add auth state loading on the client and protected route/page behavior
- [ ] Add basic auth UI (login form, logged-in state, logout action)
- [ ] Validate flow end-to-end

## Verification
- `cargo check`
- relevant `cargo run` / `dx serve` flow for the chosen target
- manual test: create or seed user, login, refresh, protected view access, logout
- verify DB file creation/migrations and failed-login handling
