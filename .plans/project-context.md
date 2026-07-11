# Claude Call Project Context

This file is the handoff document for future AI agents working on this repository. It summarizes the project goal, decisions made in the initial planning chat, V0 implementation status, and the intended direction of the product.

## Product Idea

Claude Call is intended to become a local wake-word assistant for AI coding workflows.

The final product should let the user say a wake phrase like `Claude`, automatically start voice capture through Superwhisper, and route the resulting transcription into the correct local AI agent session, especially an existing or newly created opencode session.

Intended final experience:

```text
say "Claude"
-> local wake-word detector activates
-> Superwhisper starts recording
-> transcription is sent to the right opencode session
-> the AI agent continues from the user's current project context
```

The project should feel like a hands-free bridge between voice, Superwhisper, and local AI coding agents.

## Current Repository

Repository name: `claude-call`

Remote: `https://github.com/sazuyakun/claude-call`

One-line description: `Wake your local AI workflow with a single call.`

Current implementation language: Rust.

Current V0 approach: terminal-driven fake wake-word detection through stdin.

## High-Level Architecture Direction

The architecture should stay event-pipeline based:

```text
Input Source
-> Wake Detector
-> Wake Event
-> Policy / State
-> Action Runner
-> Superwhisper / opencode / macOS automation
```

Important design rule:

```text
Detection should not know what actions happen.
Actions should not know where wake events came from.
```

That separation lets future versions replace stdin with audio wake-word detection without rewriting the action pipeline.

## Preferred Full-Scope Tech Stack

Rust should remain the main product language.

Rust responsibilities:

- daemon / long-running process
- config loading
- action orchestration
- CLI
- local IPC or HTTP API
- state machine / cooldown logic
- macOS automation wrappers

Python can be introduced later only where it helps with ML/audio model integration.

Python responsibilities later:

- wake-word model experiments
- openWakeWord integration
- microphone/model pipeline if Rust-first audio ML becomes too slow to implement

macOS automation:

- use Superwhisper deep links where available
- use AppleScript/JXA only where needed
- avoid simulated keystrokes when official URLs/API-like mechanisms exist

Suggested future crates/libraries discussed:

- Rust: `tokio`, `serde`, `toml`, `anyhow`, `thiserror`, `tracing`, `tracing-subscriber`, `clap`, `axum`, `reqwest`, `directories`
- Audio later: `cpal`, `hound`, `rubato`
- Python later: `uv`, `openwakeword`, `sounddevice`, `numpy`, `fastapi`, `pydantic`, `pytest`, `ruff`

## V0 Scope

V0 deliberately avoids real microphone wake-word detection.

V0 goal:

```text
cargo run
type claude
-> Superwhisper opens if needed and starts recording
-> app keeps listening for another trigger
```

V0 includes:

- Rust binary project
- TOML config
- stdin wake-word detector
- normalized input matching
- configured command/action runner
- Superwhisper recording action
- basic structured logs
- readable error handling
- continuous listening loop
- README documentation

V0 excludes:

- microphone input
- real wake-word detection
- Python model service
- opencode integration
- launchd daemon
- menu bar app

## V0 Task List Status

Completed:

- Create project folder/repo
- Initialize Rust app
- Add base dependencies
- Create initial module structure
- Create config directory and config file
- Define V0 config format
- Implement config loader
- Implement stdin detector
- Normalize wake input with `trim().to_lowercase()`
- Implement action runner
- Add Superwhisper action
- Wire main flow
- Keep app listening after each trigger
- Add readable errors with `anyhow`, `context`, `with_context`, and `bail`
- Add basic logs with `tracing`
- Add README V0 usage
- Add `.gitignore`
- Run formatting and compile checks
- Smoke test the V0 flow

Still worth manually confirming:

- Quit Superwhisper completely.
- Run `cargo run`.
- Type `claude`.
- Confirm Superwhisper opens and starts recording.
- Type `claude` again.
- Confirm recording can be triggered again without restarting Claude Call.

## Current File Structure

```text
claude-call/
├── .gitignore
├── .plans/
│   └── project-context.md
├── Cargo.lock
├── Cargo.toml
├── README.md
├── config/
│   └── claude-call.toml
└── src/
    ├── actions.rs
    ├── config.rs
    ├── detector.rs
    └── main.rs
```

The `.plans` folder should contain only this file unless the user explicitly asks for more files.

## Current Runtime Behavior

Run:

```bash
cargo run
```

Expected logs:

```text
loaded config wake_word=claude actions=1
listening for wake word
>
```

Type:

```text
claude
```

Expected behavior:

- stdin detector matches the wake word
- action runner runs configured action
- Superwhisper opens if needed
- Superwhisper starts recording via official deep link
- Claude Call returns to listening

## Current Config

File: `config/claude-call.toml`

```toml
wake_word = "claude"

[[actions]]
name = "start-superwhisper-recording"
command = "osascript"
args = [
  "-e",
  "tell application \"System Events\" to set superwhisperIsRunning to exists process \"superwhisper\"",
  "-e",
  "if not superwhisperIsRunning then",
  "-e",
  "tell application \"superwhisper\" to activate",
  "-e",
  "delay 1",
  "-e",
  "end if",
  "-e",
  "open location \"superwhisper://record\"",
]
```

Why it is written this way:

- Direct `open superwhisper://record` works when Superwhisper is already running.
- If Superwhisper is quit, direct deep-link opening may only launch the app and not start recording.
- The AppleScript checks if the app is running; if not, it opens Superwhisper, waits briefly, then calls the official `superwhisper://record` deep link.

Important docs reference:

- Superwhisper supports deep links:
- `superwhisper://mode?key=YOUR_MODE_KEY`
- `superwhisper://record`
- Source checked during development: `https://superwhisper.com/docs/modes/switching-modes#deep-links`

## Important Rust Concepts Discussed

`tracing_subscriber::fmt::init()`:

- Initializes terminal logging.
- Without it, `tracing::info!` and `tracing::debug!` events usually do not print.

`tracing` field syntax:

- `%value` uses `Display` formatting.
- `?value` uses `Debug` formatting.

Example:

```rust
tracing::info!(wake_word = %config.wake_word, "loaded config");
```

`?` operator:

- Used to return early if a `Result` is an error.
- Example: `Config::load_from_file("config/claude-call.toml")?`

`Ok(())`:

- Means success with no meaningful return value.
- Used because functions like `main` and action/detector helpers return `Result<()>`.

Borrowing actions:

- `for action in &config.actions` iterates by reference.
- This avoids moving/consuming `config.actions`.

Config style:

- `Config` has a struct plus an `impl` because config is a domain object and `load_from_file` is behavior attached to it.
- `detector.rs` started with a direct `pub fn` because the stdin detector is currently stateless and simple.

TOML action array:

- `[[actions]]` means `actions` is an array of tables.
- `name`, `command`, and `args` belong to one action item.
- Multiple `[[actions]]` blocks would run in order.

`with_context` vs `context`:

- Use `.context("static message")` for static errors.
- Use `.with_context(|| format!(...))` when the message includes dynamic data or should be computed lazily.

Command execution errors:

- `.status().with_context(...)?` handles failure to start the command.
- `if !status.success() { bail!(...) }` handles commands that start but exit unsuccessfully.

## Current Code Responsibilities

`src/config.rs`:

- Defines `Config` and `ActionConfig`.
- Loads TOML from disk.
- Adds read/parse context to errors.

`src/detector.rs`:

- Implements `wait_for_wake_word`.
- Prints a prompt.
- Reads stdin.
- Normalizes input.
- Returns `Ok(())` once the wake word is detected.
- Returns an error if stdin closes.

`src/actions.rs`:

- Implements `run_actions`.
- Runs configured commands in order.
- Logs action start/completion.
- Fails fast if a command cannot start or exits unsuccessfully.

`src/main.rs`:

- Initializes logging.
- Loads config.
- Logs loaded config/actions.
- Loops forever:
- waits for the wake word
- runs configured actions
- listens again

## Git / Commit History Highlights

Important commits created during V0:

- `8e48e6d Initialize V0 Rust project structure`
- `d9a4350 Add config loader`
- `f009d33 Add stdin wake detector`
- `2261724 Run configured actions`
- `c07bc1a Start Superwhisper recording`
- `42ac65b Use Superwhisper record deep link`
- `d8b5ff6 Handle Superwhisper launch before recording`
- `490b9ce Keep listening after wake actions`
- `3127490 Document V0 usage`
- `043e495 Add future scope to README`
- `0e0445b Ignore local capture files`

The user asked to keep commits small and to keep pushing after each completed code/documentation change.

## Future Product Scope

The desired final product is not just a script that opens Superwhisper. It should become a local assistant orchestration layer.

Future capabilities may include:

- real wake-word detection from microphone input
- a background daemon instead of a foreground terminal process
- launch-on-login via macOS `launchd`
- local CLI commands like `status`, `trigger`, `config check`, and `doctor`
- routing transcription to an existing opencode session
- creating an opencode session if none exists
- choosing the right project/session context automatically
- supporting multiple detector backends
- adding cooldown/debounce behavior to prevent duplicate triggers
- structured logs and better diagnostics
- optional Python wake-word model service for openWakeWord or similar

## Suggested Next Steps After V0

The next logical engineering steps are:

- Add config validation, especially requiring at least one action.
- Add a CLI flag for selecting config path.
- Add a `trigger` mode for non-interactive/manual testing.
- Add a small state/cooldown layer.
- Move toward a daemon/CLI split.
- Add opencode-focused actions once the desired terminal/session behavior is defined.

Do not jump directly into ML wake-word detection until the local app/daemon/action architecture is stable.

## Development Commands

Run app:

```bash
cargo run
```

Debug logs:

```bash
RUST_LOG=debug cargo run
```

Checks:

```bash
cargo fmt --check
cargo check
```

Smoke test one trigger:

```bash
printf 'claude\n' | cargo run
```

Smoke test repeated triggers:

```bash
printf 'claude\nclaude\n' | cargo run
```

The repeated piped test will eventually end with `Error: stdin closed`, which is expected because piped stdin ends.

## User Preferences For Future Agents

- Keep changes small and focused.
- Commit and push after each completed step.
- Explain every code change after making it.
- Prefer simple infrastructure before AI/ML complexity.
- Use Rust as the main learning/project language.
- Use Python later only if it materially simplifies wake-word ML.
- Keep V0 and docs crisp; avoid overbuilding.
- Preserve the established pattern of verifying with `cargo fmt --check` and `cargo check`.
