# Claude Call

Wake your local AI workflow with a single call.

## Future Scope

The final product is a small local assistant daemon that listens for a wake phrase, starts voice capture, and routes the result into the active AI coding workflow.

The intended experience:

```text
say "Claude"
-> Superwhisper starts recording
-> transcription is sent to the right opencode session
-> the AI agent continues from your current project context
```

Long term, Claude Call should feel like a hands-free bridge between your voice, Superwhisper, and local agent sessions.

## V0

V0 is a terminal-driven prototype. It does not listen to the microphone yet.

Current behavior:

```text
type claude
-> detect wake word
-> start Superwhisper recording
-> keep listening
```

## Requirements

- macOS
- Rust toolchain
- Superwhisper installed

## Run Interactive Mode

```bash
cargo run
```

When the prompt appears, type:

```text
claude
```

Expected result:

```text
Superwhisper opens if needed and starts recording.
```

The app keeps listening after each trigger, so you can type `claude` again to start another recording.

## CLI Usage

Show available commands and options:

```bash
cargo run -- --help
```

Use a custom config path:

```bash
cargo run -- --config config/claude-call.toml
```

Run the wake listener in the foreground terminal:

```bash
cargo run -- foreground
```

Running without a subcommand currently does the same thing:

```bash
cargo run
```

Run daemon mode:

```bash
cargo run -- daemon
```

Daemon mode currently runs in the attached terminal. It is not installed as a background service yet, but it starts the local control API on `127.0.0.1:8765`.

Ask a running daemon for status:

```bash
cargo run -- status
```

Expected response:

```json
{"status":"ok"}
```

Ask the running daemon to trigger the configured actions:

```bash
cargo run -- trigger
```

Route transcript text through the running daemon:

```bash
cargo run -- transcript "open the current file and explain the main function"
```

Expected response:

```json
{"status":"routed"}
```

Route transcript text directly without using the daemon:

```bash
cargo run -- transcript --direct "open the current file and explain the main function"
```

Run the configured actions directly in the current process without using the daemon:

```bash
cargo run -- trigger --direct
```

Validate the config without listening or running actions:

```bash
cargo run -- config check
```

Validate a custom config file:

```bash
cargo run -- --config path/to/claude-call.toml config check
```

## Config

The default config lives at:

```text
config/claude-call.toml
```

Current config:

```toml
wake_word = "claude"
cooldown_seconds = 5

[wake_detector]
backend = "stdin"

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

Transcript routing is optional, but transcript commands require it. Configure it only when you know which opencode project and session should receive voice text:

```toml
[routing.opencode]
project_path = "/Users/you/Coding/apps/example-project"
session_id = "ses_your_known_opencode_session_id"
```

Optional routing fields:

```toml
[routing.opencode]
project_path = "/Users/you/Coding/apps/example-project"
session_id = "ses_your_known_opencode_session_id"
command = "opencode"
agent = "build"
```

Claude Call routes by running:

```bash
opencode run --session <session_id> --dir <project_path> "<transcript>"
```

If `agent` is configured, `--agent <agent>` is included. If `command` is omitted, Claude Call uses `opencode` from `PATH`.

The action checks whether Superwhisper is already running. If not, it opens the app, waits briefly, then calls Superwhisper's official record deep link.

`cooldown_seconds` controls how soon another wake event can run actions after the last accepted wake event. If `cooldown_seconds = 5`, a second `claude` typed immediately after the first one is ignored. After 5 seconds, another `claude` can run actions again.

`wake_detector.backend` controls where wake events come from. Supported values:

- `stdin`: type the wake word in the terminal. This is the default when `[wake_detector]` is omitted.
- `microphone`: reserved for the Phase 8 Python/audio backend. It is accepted by config but fails clearly at runtime for now.

Config is validated at startup. Claude Call currently requires:

- a non-empty `wake_word`
- `cooldown_seconds` greater than `0`
- `wake_detector.backend` must be `stdin` or `microphone` when set
- at least one action
- non-empty action names
- non-empty action commands
- if routing is configured, non-empty `routing.opencode.project_path`
- if routing is configured, non-empty `routing.opencode.session_id`
- if routing command or agent overrides are configured, non-empty values

## Logs

Normal logs:

```bash
cargo run
```

Debug logs:

```bash
RUST_LOG=debug cargo run
```

Daemon control logs are emitted by the daemon process. Useful events include:

- control server startup
- status request received
- trigger request received
- trigger completed or failed
- transcript request received
- transcript routed to opencode, or rejected with a clear routing error

## Transcription Flow

Phase 5 treats a completed transcription as a local text payload entering Claude Call. Superwhisper is still responsible for recording and transcription; Claude Call receives the finished text through either a direct CLI command or the daemon's localhost-only control API.

Current local ingest shape:

```http
POST http://127.0.0.1:8765/transcript
Content-Type: application/json

{"text":"open the current file and explain the main function"}
```

Claude Call validates that transcript text is not empty, then Phase 6 routes it to the explicitly configured opencode project and session.

## opencode Routing

Phase 6 intentionally avoids guessing the active terminal or current opencode workspace. A transcript can contain private local context, so Claude Call only routes when config names a project path and session id.

Safe behavior:

- no routing config means transcript commands fail with a clear error
- ambiguous routing is not attempted
- daemon transcript ingest uses the daemon's loaded config, not the caller's current directory
- direct transcript routing loads and validates the selected config before sending text

Current route command:

```bash
opencode run --session <session_id> --dir <project_path> "<transcript>"
```

Use `opencode session` or the opencode UI to identify the session you want, then copy that known session id into `routing.opencode.session_id`.

Superwhisper assumptions:

- recording still starts through Superwhisper's `superwhisper://record` deep link
- transcript completion should use an official Superwhisper handoff if available, such as a post-transcription command, webhook, shortcut, or app integration
- if no official completion hook is available, a small local bridge can call `POST /transcript`
- simulated keystrokes are intentionally avoided for transcript handoff

## Wake Detection

Phase 7 introduces an explicit detector backend boundary. The daemon, cooldown policy, actions, and transcript routing all consume the same `WakeEvent` regardless of where it came from.

Current behavior:

- default config uses `stdin`
- typing `claude` still produces the wake event
- `microphone` is a named backend boundary, not an implemented audio runtime yet
- real Python/audio integration is Phase 8
- custom wake model training is Phase 9

Example explicit stdin config:

```toml
[wake_detector]
backend = "stdin"
```

Example future microphone config:

```toml
[wake_detector]
backend = "microphone"
```

With `microphone` today, the app exits with:

```text
microphone wake detector backend is planned for Phase 8
```

## Verify

Run the checks:

```bash
cargo fmt --check
cargo check
```

Check config only:

```bash
cargo run -- config check
```

Direct trigger test:

```bash
cargo run -- trigger --direct
```

Daemon status and trigger test:

```bash
cargo run -- daemon
```

In another terminal:

```bash
cargo run -- status
cargo run -- trigger
cargo run -- transcript "summarize this project"
```

Direct transcript test:

```bash
cargo run -- transcript --direct "summarize this project"
```

The transcript commands require `[routing.opencode]` in the selected config.

Manual V0 test:

```text
1. Quit Superwhisper.
2. Run cargo run.
3. Type claude.
4. Confirm Superwhisper opens and starts recording.
5. Type claude again.
6. If you type it inside the cooldown window, confirm the wake event is ignored.
7. Wait for the cooldown to pass.
8. Type claude again.
9. Confirm it starts recording again without restarting the app.
```

Microphone backend boundary test:

```toml
[wake_detector]
backend = "microphone"
```

Running `cargo run -- --config path/to/microphone-config.toml foreground` should fail clearly because microphone wake detection is intentionally deferred to Phase 8.

## Notes

- V0 uses stdin as the fake wake-word detector.
- `[wake_detector]` defaults to `backend = "stdin"` when omitted.
- `backend = "microphone"` is a Phase 7 boundary only; Python/audio runtime comes in Phase 8.
- V0 uses Superwhisper's `superwhisper://record` deep link to start recording.
- `daemon` runs the current long-lived wake listener and local control API in the attached terminal.
- `status` calls the daemon over localhost HTTP.
- `trigger` asks the daemon to run configured actions.
- `trigger --direct` runs configured actions immediately in the current process and exits.
- `transcript` sends transcript text to the daemon over localhost HTTP for opencode routing.
- `transcript --direct` routes transcript text to opencode in the current process, then exits.
- `config check` validates config without running actions.
- Interactive wake detection uses cooldown state; daemon/manual trigger bypasses wake detection and cooldown.
- Transcript routing requires an explicit opencode project path and session id; Claude Call does not infer the active workspace.
- Real microphone wake-word detection is intentionally out of scope for V0.
