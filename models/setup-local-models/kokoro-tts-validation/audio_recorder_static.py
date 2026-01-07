#!/usr/bin/env python3
"""
Static audio recorder node for Kokoro TTS timing analysis.
"""

import time
import numpy as np
import soundfile as sf
import pyarrow as pa
from dora import Node
import json
from pathlib import Path


def main():
    """Main function for static audio recorder node."""

    print("=" * 80)
    print("Static Audio Recorder - Kokoro TTS Timing Analysis")
    print("=" * 80)

    node = Node()

    # Create output directory
    output_dir = Path("tts_output")
    output_dir.mkdir(exist_ok=True)
    print(f"Output directory: {output_dir.absolute()}")

    # Statistics tracking
    total_segments = 0
    total_audio_duration = 0
    start_time = None
    audio_segments = []
    sample_rate = 24000  # Kokoro default

    print("\nWaiting for audio from kokoro-tts...")
    print("-" * 60)

    for event in node:
        if event["type"] == "INPUT":
            if event["id"] == "audio":
                # Handle float array - convert to numpy
                audio_data = event["value"].to_numpy()
                metadata = event.get("metadata", {})

                # First audio received
                if start_time is None:
                    start_time = time.time()
                    print(f"⏱️  First audio received at {time.strftime('%H:%M:%S')}")

                # Check if it's empty (error case)
                if len(audio_data) == 0:
                    print(f"❌ Received empty audio")
                    continue

                # Process audio segment
                total_segments += 1
                sample_rate = metadata.get("sample_rate", 24000)
                duration = len(audio_data) / sample_rate
                total_audio_duration += duration

                audio_segments.append(audio_data)

                print(f"Segment {total_segments}: {duration:.3f}s ({len(audio_data)} samples @ {sample_rate}Hz)")

        elif event["type"] == "STOP":
            break

    # Save results
    if audio_segments:
        # Concatenate all audio
        full_audio = np.concatenate(audio_segments)
        output_file = output_dir / "kokoro_test_output.wav"
        sf.write(output_file, full_audio, sample_rate)
        print(f"\n✓ Saved audio to: {output_file}")
        print(f"  Total duration: {total_audio_duration:.3f}s")
        print(f"  Total segments: {total_segments}")
        print(f"  Sample rate: {sample_rate}Hz")

        if start_time:
            total_time = time.time() - start_time
            rtf = total_audio_duration / total_time if total_time > 0 else 0
            print(f"  Processing time: {total_time:.3f}s")
            print(f"  Real-time factor: {rtf:.2f}x")
    else:
        print("\n⚠️  No audio received")

    # Save timing results
    results = {
        "total_segments": total_segments,
        "total_audio_duration": total_audio_duration,
        "processing_time": time.time() - start_time if start_time else 0,
        "sample_rate": sample_rate
    }

    with open(output_dir / "kokoro_timing_results.json", "w") as f:
        json.dump(results, f, indent=2)

    print("\n" + "=" * 80)
    print("Audio recorder completed")
    print("=" * 80)


if __name__ == "__main__":
    main()
