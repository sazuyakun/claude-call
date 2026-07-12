# AGENTS.md

Instructions for coding agents working in this repository. Treat this as the agent-focused companion to `README.md`.

## Project Overview

- Claude Call is a Rust-first local assistant for voice-driven AI coding workflows.
- V0 is a terminal prototype that detects `claude` from stdin and starts Superwhisper recording.
- The intended final product is a background daemon plus CLI that routes voice transcriptions into the right opencode session.
- Planning context lives in `.plans/`, starting with `.plans/v0-context.md` and `.plans/v1-phases.md`.

## Setup Commands

- Build/check the project: `cargo check`
- Run the app: `cargo run`
- Run with debug logs: `RUST_LOG=debug cargo run`
- Format check: `cargo fmt --check`

## Testing Instructions

- Run `cargo fmt --check` and `cargo check` before calling code changes complete.
- Use `cargo run` for the interactive V0 flow.
- Use `printf 'claude\n' | cargo run` for a one-trigger smoke test.
- The piped stdin smoke test may end with `stdin closed`; that is expected after input ends.

## Code Style

- Keep Rust as the main product language.
- Keep code clean, direct, and simple.
- Follow first principles: understand the data flow before adding abstractions.
- Prefer readable domain names over generic helper names.
- Use `anyhow` for application-level errors and add context at boundaries.
- Keep modules small and responsibility-focused.
- Add comments only when they explain non-obvious behavior or platform assumptions.

## Architecture Guidelines

- Keep the event pipeline explicit: input source, wake detector, wake event, policy/state, action runner.
- Detection should not know what actions happen.
- Actions should not know where wake events came from.
- Policy/state should decide whether a wake event should trigger actions.
- Config should be validated early with clear errors.
- Logs should explain important decisions without becoming noisy.

## Product Constraints

- Do not add a framework or service until the product needs it.
- Do not introduce Python unless wake-word ML/audio integration clearly benefits from it.
- Avoid simulated keystrokes when an official URL, local API, or explicit command works.
- Prefer small focused changes over broad rewrites.
- Keep README instructions current when behavior changes.

## Security And Privacy

- Treat voice recordings, transcripts, and local project paths as sensitive local data.
- Do not commit generated audio captures, transcripts, local logs, secrets, or machine-specific credentials.
- Prefer local-only APIs or IPC for daemon control unless there is a clear reason to expose anything beyond localhost.

## Commit And Handoff Notes

- After completing each code change from a prompt, commit and push the finished change.
- Keep commits small and focused when asked to commit.
- Explain every meaningful behavior change after making it.
- Update `.plans/` when phase plans or product direction change.
- If instructions conflict, follow the closest `AGENTS.md`; explicit user prompts override this file.
