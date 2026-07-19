#!/usr/bin/env python3
"""Python wake backend process for Claude Call.

The Rust daemon owns policy, actions, routing, and process lifecycle. This
process owns the Python-side wake backend and emits the shared wake event
protocol on stdout when detection succeeds.
"""

from __future__ import annotations

import argparse
import os
import sys


def main() -> int:
    parser = argparse.ArgumentParser(description="Claude Call Python wake backend")
    parser.add_argument("--wake-word", required=True)
    args = parser.parse_args()

    forced_event = os.environ.get("CLAUDE_CALL_WAKE_EVENT")
    if forced_event:
        print(forced_event, flush=True)
        return 0

    print(
        (
            "python wake backend started for "
            f"{args.wake_word!r}, but no wake model is configured yet"
        ),
        file=sys.stderr,
        flush=True,
    )
    print(
        "Phase 9 wires a temporary pre-existing model into this backend.",
        file=sys.stderr,
        flush=True,
    )
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
