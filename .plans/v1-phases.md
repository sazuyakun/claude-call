# Claude Call V1 Phases

V1 is the path from the current terminal prototype to the final product experience.

The end result should be a local assistant daemon that listens for the wake phrase, starts Superwhisper voice capture, receives or routes the resulting transcription, and sends it to the right local AI coding session.

Intended final experience:

```text
say "Claude"
-> local wake-word detector activates
-> Superwhisper starts recording
-> transcription is sent to the right opencode session
-> the AI agent continues from the user's current project context
```

## V1 Principles

- Keep Rust as the main product language.
- Keep the event pipeline explicit and simple.
- Keep detection, policy, and actions separated.
- Prefer local, inspectable mechanisms before clever automation.
- Build stable app/daemon infrastructure before ML complexity.
- Add Python only if it materially simplifies wake-word model integration.
- Avoid broad abstractions until there are at least two real implementations.

Core architecture:

```text
Input Source
-> Wake Detector
-> Wake Event
-> Policy / State
-> Action Runner
-> Superwhisper / opencode / macOS automation
```

## Phase 1: Productize The Current CLI

Goal: make the current V0 behavior reliable enough to become the base for daemon work.

Scope:

- Add CLI flags for config path and one-shot trigger mode.
- Add config validation with clear errors.
- Add a `doctor` or `config check` command if it stays simple.
- Keep stdin wake detection as a supported development backend.
- Preserve the current Superwhisper recording action.

Exit criteria:

- `cargo run` still supports the V0 manual flow.
- A manual trigger can run without interactive stdin.
- Invalid config fails early with readable output.
- `cargo fmt --check` and `cargo check` pass.

Phase 1 commit plan:

| Done | Step | Commit goal                         | What changes                                                                                                                                      | Verification                                                                                  |
| ---- | ---- | ----------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| [x]  | 1    | Add CLI argument parsing foundation | Add a small CLI layer with `clap`, keep default behavior as the current interactive stdin mode, and avoid changing runtime behavior yet.          | `cargo fmt --check`, `cargo check`, `cargo run`                                               |
| [x]  | 2    | Add configurable config path        | Add a `--config <path>` flag and route config loading through it instead of hardcoding `config/claude-call.toml`.                                 | `cargo fmt --check`, `cargo check`, `cargo run -- --config config/claude-call.toml`           |
| [x]  | 3    | Add config validation               | Validate config after loading, starting with required non-empty `wake_word`, at least one action, non-empty action names, and non-empty commands. | `cargo fmt --check`, `cargo check`, temporarily test invalid config errors if useful          |
| [x]  | 4    | Add manual trigger mode             | Add a one-shot command or flag that runs the configured actions without waiting for stdin wake input.                                             | `cargo fmt --check`, `cargo check`, `cargo run -- trigger` or chosen command shape            |
| [x]  | 5    | Add config check command            | Add a command that loads and validates config without running detectors or actions.                                                               | `cargo fmt --check`, `cargo check`, `cargo run -- config check` or chosen command shape       |
| [x]  | 6    | Refresh README Phase 1 usage        | Document the new CLI usage, config path option, trigger mode, and config check command.                                                           | Read README for accuracy, run listed commands if practical                                    |
| [x]  | 7    | Phase 1 final smoke test            | Run the core flows and fix any rough edges before marking Phase 1 complete.                                                                       | `cargo fmt --check`, `cargo check`, `cargo run`, manual trigger command, config check command |

## Phase 2: State And Cooldown

Goal: prevent noisy repeated triggers and make behavior predictable.

Scope:

- Add a small state/policy layer between wake detection and actions.
- Add cooldown/debounce config.
- Track whether a trigger is accepted, ignored, or rejected.
- Log state decisions clearly.

Exit criteria:

- Repeated triggers inside the cooldown window do not run actions repeatedly.
- Logs explain why a trigger was accepted or ignored.
- Detection code does not know about cooldown rules.
- Action code does not know where wake events came from.

Phase 2 commit plan:

| Done | Step | Commit goal                     | What changes                                                                                                                   | Verification                                                                                 |
| ---- | ---- | ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------- |
| [x]  | 1    | Add wake event type             | Introduce a small `WakeEvent` domain type so detectors return an event instead of only `Ok(())`.                               | `cargo fmt --check`, `cargo check`, stdin smoke test                                         |
| [x]  | 2    | Add policy/state module         | Add a policy layer that receives wake events and decides whether actions should run, initially accepting every event.           | `cargo fmt --check`, `cargo check`, stdin smoke test                                         |
| [x]  | 3    | Add cooldown config             | Add a config value for cooldown duration with a simple default, and validate that it is sensible.                              | `cargo fmt --check`, `cargo check`, `cargo run -- config check`                              |
| [x]  | 4    | Enforce cooldown decisions      | Track the last accepted trigger and ignore repeated wake events inside the cooldown window.                                    | `cargo fmt --check`, `cargo check`, repeated stdin trigger smoke test                        |
| [x]  | 5    | Log accepted and ignored events | Add clear logs explaining when wake events are accepted or ignored and why.                                                    | `cargo fmt --check`, `cargo check`, inspect logs during repeated trigger smoke test          |
| [x]  | 6    | Update docs for cooldown        | Document cooldown behavior and config in README and keep the phase plan current.                                               | Read README for accuracy, run `cargo run -- config check`                                    |
| [x]  | 7    | Phase 2 final smoke test        | Run final checks for default mode, repeated triggers, `trigger`, and `config check`; fix rough edges before completing Phase 2. | `cargo fmt --check`, `cargo check`, repeated trigger, manual trigger, config check commands |

## Phase 3: Daemon And CLI Split

Goal: move from a foreground terminal process to a local background service while keeping a small CLI for control.

Scope:

- Split the app into daemon runtime and CLI commands.
- Add local IPC or HTTP on localhost for commands such as `status` and `trigger`.
- Keep the daemon long-running and observable.
- Keep config loading and action orchestration in shared Rust code.

Exit criteria:

- The daemon can run continuously without an attached terminal.
- The CLI can ask for status and manually trigger the action pipeline.
- Failures are logged with enough context to debug locally.
- The old foreground development mode still exists if useful.

Local control decision:

- Use a localhost-only HTTP control API for V1 daemon/CLI communication.
- Bind only to `127.0.0.1`, never `0.0.0.0`.
- Start with a small fixed control surface: `status` and `trigger`.
- Keep request/response payloads simple JSON when a body is needed.
- Prefer clear errors if no daemon is running instead of silently falling back, except where a command explicitly supports direct local execution.
- Do not add authentication for the first local-only version; revisit before exposing anything beyond loopback or adding sensitive transcript routes.
- Likely implementation path: Rust HTTP server in daemon mode and Rust HTTP client in CLI mode, using crates such as `axum` and `reqwest` when this step is implemented.
- Keep action orchestration inside the daemon process so the CLI asks for work rather than duplicating daemon state.

Phase 3 commit plan:

| Done | Step | Commit goal                         | What changes                                                                                                                   | Verification                                                                                  |
| ---- | ---- | ----------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------- |
| [x]  | 1    | Extract app runtime                 | Move config loading, action logging, trigger handling, and interactive wake loop out of `main.rs` into reusable runtime code.   | `cargo fmt --check`, `cargo check`, `cargo run -- config check`, safe stdin smoke test        |
| [x]  | 2    | Reorganize source modules           | Group app runtime/config/actions and wake detector/event/policy into clearer module folders before adding daemon code.          | `cargo fmt --check`, `cargo check`, `cargo run -- config check`, safe stdin smoke test        |
| [x]  | 3    | Clarify foreground command          | Add an explicit foreground/run command shape while preserving the current default interactive behavior.                        | `cargo fmt --check`, `cargo check`, `cargo run -- --help`, safe stdin smoke test              |
| [x]  | 4    | Add daemon command shell            | Add a `daemon` subcommand that runs the same long-running foreground runtime for now, without installing a service yet.         | `cargo fmt --check`, `cargo check`, `cargo run -- daemon` smoke test with safe config         |
| [x]  | 5    | Choose local control mechanism      | Document and encode the decision for local control, likely localhost HTTP or local IPC, before implementing daemon control.     | Review `.plans/v1-phases.md`; no runtime behavior change                                      |
| [x]  | 6    | Add daemon status endpoint/command  | Add the smallest local `status` path so the CLI can ask a running daemon whether it is alive.                                  | `cargo fmt --check`, `cargo check`, run daemon and status command                             |
| [x]  | 7    | Replace control server with axum    | Replace the manual HTTP parsing/writing in daemon control with declarative `axum` routes while keeping `/status` behavior.      | `cargo fmt --check`, `cargo check`, run daemon and status command                             |
| [x]  | 8    | Add daemon trigger command path     | Route CLI trigger requests to the daemon when available, while preserving direct local trigger behavior for development.        | `cargo fmt --check`, `cargo check`, safe daemon trigger test                                  |
| [x]  | 9    | Improve daemon observability        | Ensure startup, config, status, trigger, and shutdown paths log clear useful information without becoming noisy.                | Inspect logs during daemon/status/trigger smoke tests                                         |
| [x]  | 10   | Update README for daemon CLI split  | Document foreground mode, daemon mode, status, trigger behavior, and local-only control assumptions.                           | Read README for accuracy; run documented non-destructive commands                             |
| [x]  | 11   | Phase 3 final smoke test            | Run final checks for foreground mode, daemon mode, status, trigger, config check, and cooldown behavior before completing.      | `cargo fmt --check`, `cargo check`, daemon/status/trigger/config check smoke tests            |

## Phase 4: macOS Launch Setup

Goal: make Claude Call start and run like a normal local assistant.

Scope:

- Add macOS `launchd` setup documentation or an installer command.
- Support launch-on-login.
- Add commands to install, uninstall, start, stop, and restart the daemon if they stay simple.
- Document required macOS permissions for microphone, automation, and Superwhisper behavior.

Exit criteria:

- Claude Call can start on login through `launchd`.
- The user can stop and restart the daemon without editing plist files manually.
- The setup remains understandable and reversible.

## Phase 5: Superwhisper Transcription Flow

Goal: move beyond only starting recording and define how transcription leaves Superwhisper.

Decision: Phase 5 treats a completed transcription as local text input to Claude Call rather than coupling routing to Superwhisper UI automation. Claude Call exposes a localhost-only transcript ingest path and a direct CLI path so the text payload can be observed, logged, and later routed to agents. Superwhisper remains responsible for recording/transcription; the handoff should use an official app mechanism such as a post-transcription command/webhook/deep-link integration if available on the user's installed Superwhisper version. If no official completion hook is available, a small local bridge may call the Claude Call ingest endpoint, but simulated keystrokes remain out of scope.

Scope:

- Decide how Claude Call receives or retrieves the final transcription.
- Prefer official Superwhisper mechanisms if available.
- Use AppleScript/JXA only where needed.
- Avoid simulated keystrokes unless there is no better local interface.
- Normalize transcription events before routing them to agents.

Exit criteria:

- A completed voice capture can become a text payload inside Claude Call.
- The payload can be logged or inspected before routing.
- The transcription path is documented with its macOS/Superwhisper assumptions.

Phase 5 commit plan:

| Done | Step | Commit goal                         | What changes                                                                                                               | Verification                                                                                  |
| ---- | ---- | ----------------------------------- | -------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| [x]  | 1    | Document transcription path choice  | Research and document the chosen Superwhisper-to-Claude-Call transcription handoff path and assumptions.                    | Review `.plans/v1-phases.md`; no runtime behavior change                                      |
| [x]  | 2    | Add transcript domain type          | Add a small `TranscriptEvent` type so text payloads are explicit before routing work begins.                                | `cargo fmt --check`, `cargo check`                                                           |
| [x]  | 3    | Add direct transcript command       | Add a CLI command that accepts transcript text and logs the payload without needing Superwhisper yet.                       | `cargo fmt --check`, `cargo check`, direct transcript command                                 |
| [x]  | 4    | Add local transcript ingest endpoint| Add a localhost-only daemon endpoint for receiving transcript text payloads.                                                | `cargo fmt --check`, `cargo check`, daemon ingest smoke test                                  |
| [x]  | 5    | Wire CLI transcript send path       | Add a CLI path that sends transcript text to the running daemon, while keeping direct local handling available.              | `cargo fmt --check`, `cargo check`, daemon transcript send smoke test                         |
| [x]  | 6    | Document Superwhisper assumptions   | Update README with the transcript ingest shape, direct command, and current Superwhisper/macOS assumptions.                 | Read README for accuracy; run non-destructive documented commands                             |
| [x]  | 7    | Phase 5 final smoke test            | Run final checks for direct transcript, daemon ingest, config check, status, and trigger before completing Phase 5.         | `cargo fmt --check`, `cargo check`, transcript/status/trigger/config check smoke tests        |

## Phase 6: opencode Session Routing

Goal: send transcriptions to the right AI coding session.

Decision: Phase 6 routes only to an explicitly configured opencode project path and session id. Claude Call does not try to infer the active terminal, current workspace, or last opencode session because that can send private text to the wrong project. The first routing implementation shells out to `opencode run --session <session_id> --dir <project_path> <transcript>`, with an optional command override for nonstandard installs or wrappers.

Scope:

- Define how Claude Call discovers active opencode sessions.
- Route to an existing session when possible.
- Create or select a session when none is active, only after the expected behavior is clear.
- Preserve project context and avoid sending text to the wrong workspace.
- Add routing output for debugging before sending text onward.

Exit criteria:

- A transcription can be routed to a known opencode session.
- Routing decisions are explainable in logs.
- Ambiguous routing fails safely instead of guessing silently.

Phase 6 commit plan:

| Done | Step | Commit goal                    | What changes                                                                                                       | Verification                                                                           |
| ---- | ---- | ------------------------------ | ------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------- |
| [x]  | 1    | Document routing decision      | Decide the first routing contract and document how Claude Call avoids sending text to the wrong workspace.          | Review `.plans/v1-phases.md`; no runtime behavior change                               |
| [x]  | 2    | Add routing config             | Add explicit opencode routing config with validation so routing has a known project/session target.                 | `cargo fmt --check`, `cargo check`, config validation smoke tests                      |
| [x]  | 3    | Add route domain type          | Add a small routing decision/result type so transcript routing logs stay explainable.                               | `cargo fmt --check`, `cargo check`                                                    |
| [x]  | 4    | Route transcript locally       | Replace transcript log-only handling with local routing output for direct transcript commands.                      | `cargo fmt --check`, `cargo check`, direct transcript smoke test                       |
| [x]  | 5    | Route daemon transcripts       | Make daemon transcript ingest use the same routing logic and fail safely when config is ambiguous or incomplete.    | `cargo fmt --check`, `cargo check`, daemon transcript smoke test                       |
| [x]  | 6    | Document opencode assumptions  | Update README with routing config, safe failure behavior, and current opencode handoff assumptions.                 | Read README for accuracy; run non-destructive documented commands                      |
| [x]  | 7    | Phase 6 final smoke test       | Run final checks for config validation, direct routing, daemon routing, status, and trigger.                        | `cargo fmt --check`, `cargo check`, config/status/trigger/transcript smoke tests       |

## Phase 7: Real Wake-Word Detection

Goal: replace stdin with microphone-based wake detection without rewriting the action pipeline.

Scope:

- Add a microphone/audio input backend.
- Evaluate Rust-first audio options before introducing Python.
- Introduce Python/openWakeWord only if it shortens the path materially.
- Keep model/backend details behind the wake detector boundary.
- Add config for detector backend selection.

Exit criteria:

- Saying the wake phrase can produce the same wake event as stdin did.
- Stdin/manual trigger remains available for debugging.
- The daemon/action/policy layers do not need detector-specific changes.

## Phase 8: Python Wake Backend Integration

Goal: add the Python wake-word runtime path only after detector boundaries are stable.

Scope:

- Add the smallest Python bridge needed to run a proven wake-word backend locally.
- Keep Rust as the daemon, config, policy, action, and routing owner.
- Keep Python behind the wake detector backend boundary.
- Define how audio frames, wake events, errors, and process lifecycle cross the Rust/Python boundary.
- Avoid training or model customization in this phase.

Exit criteria:

- A Python-backed wake detector can emit the same wake event shape as stdin/manual detection.
- The Rust daemon can start, supervise, and stop the Python wake backend cleanly.
- Stdin/manual detection remains available for debugging and fallback.
- Python setup and local privacy assumptions are documented.

## Phase 9: Custom Wake Model Training

Goal: train and evaluate the real Claude Call wake model after the Python runtime path is working.

Scope:

- Define the target wake phrase, dataset shape, and local data handling rules.
- Add a minimal training/evaluation workflow for the chosen wake-word backend.
- Track false accepts, false rejects, latency, CPU usage, and battery impact.
- Keep generated datasets, recordings, and model artifacts out of git unless explicitly release-ready.
- Add config for selecting a trained local model.

Exit criteria:

- A trained local wake model can trigger Claude Call reliably enough for daily testing.
- Evaluation results are documented with clear caveats.
- Model files are stored and loaded through an explicit local path.
- Privacy rules for voice samples and trained artifacts are documented.

## Phase 10: Final Polish

Goal: make V1 feel like a finished local product.

Scope:

- Add clear setup, troubleshooting, and verification docs.
- Improve diagnostics and structured logs.
- Add tests around config, policy, and routing where practical.
- Add safe defaults for wake phrase, cooldown, and routing behavior.
- Keep the product small: daemon, CLI, config, docs.

Exit criteria:

- A new user can install, configure, run, and debug Claude Call from the README.
- The full flow works from wake phrase to routed transcription.
- Failure modes are understandable and recoverable.
- V1 does not depend on hidden manual steps.

## Recommended V1 Order

1. Productize the current CLI.
2. Add state and cooldown.
3. Split daemon and CLI.
4. Add macOS launch setup.
5. Define Superwhisper transcription output.
6. Add opencode routing.
7. Add real wake-word detection.
8. Add Python wake backend integration.
9. Train and evaluate the custom wake model.
10. Polish docs, diagnostics, and tests.

Do not start with microphone ML. The product becomes easier to finish if the daemon, config, action pipeline, routing behavior, and detector backend boundary are stable first.
