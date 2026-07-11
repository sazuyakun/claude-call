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

## Run

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

## Config

V0 config lives at:

```text
config/claude-call.toml
```

Current config:

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

The action checks whether Superwhisper is already running. If not, it opens the app, waits briefly, then calls Superwhisper's official record deep link.

## Logs

Normal logs:

```bash
cargo run
```

Debug logs:

```bash
RUST_LOG=debug cargo run
```

## Verify

Run the checks:

```bash
cargo fmt --check
cargo check
```

Manual V0 test:

```text
1. Quit Superwhisper.
2. Run cargo run.
3. Type claude.
4. Confirm Superwhisper opens and starts recording.
5. Type claude again.
6. Confirm it starts recording again without restarting the app.
```

## Notes

- V0 uses stdin as the fake wake-word detector.
- V0 uses Superwhisper's `superwhisper://record` deep link to start recording.
- Real microphone wake-word detection is intentionally out of scope for V0.
