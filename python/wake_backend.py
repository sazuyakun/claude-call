#!/usr/bin/env python3
"""Python wake backend process for Claude Call.

The Rust daemon owns policy, actions, routing, and process lifecycle. This
process owns the Python-side wake backend and emits the shared wake event
protocol on stdout when detection succeeds.
"""

from __future__ import annotations

import argparse
import os
import queue
import sys
from typing import NoReturn


SAMPLE_RATE = 16_000
FRAME_MS = 80
FRAME_SAMPLES = SAMPLE_RATE * FRAME_MS // 1000


def fail(message: str) -> NoReturn:
    print(message, file=sys.stderr, flush=True)
    raise SystemExit(2)


def main() -> int:
    parser = argparse.ArgumentParser(description="Claude Call Python wake backend")
    parser.add_argument("--wake-word", required=True)
    parser.add_argument("--model", required=True)
    parser.add_argument("--threshold", required=True, type=float)
    args = parser.parse_args()

    forced_event = os.environ.get("CLAUDE_CALL_WAKE_EVENT")
    if forced_event:
        print(forced_event, flush=True)
        return 0

    try:
        import numpy as np
        import sounddevice as sd
        import openwakeword
        from openwakeword.model import Model
    except ImportError as error:
        fail(
            "missing Python wake dependencies; run "
            "python3 -m pip install -r python/requirements.txt "
            f"({error})"
        )

    if not 0 <= args.threshold <= 1:
        fail("--threshold must be between 0 and 1")

    try:
        openwakeword.utils.download_models()
        model = Model(wakeword_models=[args.model])
    except Exception as error:  # noqa: BLE001 - report backend setup errors to Rust.
        fail(f"failed to initialize openWakeWord model {args.model!r}: {error}")

    audio_frames: queue.Queue[bytes] = queue.Queue()

    def on_audio(indata, frames, time, status):  # noqa: ANN001 - sounddevice callback API.
        if status:
            print(f"audio input status: {status}", file=sys.stderr, flush=True)
        audio_frames.put(bytes(indata))

    try:
        with sd.RawInputStream(
            samplerate=SAMPLE_RATE,
            blocksize=FRAME_SAMPLES,
            dtype="int16",
            channels=1,
            callback=on_audio,
        ):
            print(
                (
                    "listening for existing wake model "
                    f"{args.model!r} for configured wake word {args.wake_word!r}"
                ),
                file=sys.stderr,
                flush=True,
            )

            while True:
                frame = np.frombuffer(audio_frames.get(), dtype=np.int16)
                prediction = model.predict(frame)
                score = float(prediction.get(args.model, 0.0))

                if score >= args.threshold:
                    print("wake", flush=True)
                    return 0
    except KeyboardInterrupt:
        return 130
    except Exception as error:  # noqa: BLE001 - report backend runtime errors to Rust.
        fail(f"python wake backend failed: {error}")


if __name__ == "__main__":
    raise SystemExit(main())
