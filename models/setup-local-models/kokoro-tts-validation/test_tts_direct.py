#!/usr/bin/env python3
"""
Direct test of Kokoro TTS with timing and audio file saving.
"""

import sys
import time
import os
import argparse
import numpy as np
import soundfile as sf
from pathlib import Path


def test_tts_timing(language='en', voice='af_heart', text=None):
    """Test Kokoro TTS with text and save audio.

    Args:
        language: Language code ('en', 'zh', 'ja', 'ko')
        voice: Voice name to use
        text: Custom text to synthesize (optional)
    """

    print("=" * 80)
    print("Direct Kokoro TTS Timing Test")
    print("=" * 80)

    # Default test texts
    if text is None:
        if language in ['zh', 'chinese']:
            text = """我们说中国式现代化是百年大战略，这又分为三个阶段。第一个阶段，我们先用30年时间建成了独立完整的工业体系和国民经济体系；再用40年，到2021年，全面建成了小康社会。我们现在正处于第三个阶段，这又被分成上下两篇：上半篇是到2035年基本实现社会主义现代化；下半篇是到本世纪中叶，也就是2050年，建成社会主义现代化强国。"""
        else:
            text = """Artificial intelligence is revolutionizing the way we interact with technology. From voice assistants to autonomous vehicles, AI systems are becoming increasingly sophisticated. Machine learning algorithms can now process vast amounts of data, recognize patterns, and make decisions with remarkable accuracy. This transformation is reshaping industries worldwide."""

    print(f"\nLanguage: {language}")
    print(f"Voice: {voice}")
    print(f"Text: {len(text)} characters")
    print("-" * 60)
    print(text)
    print("-" * 60)

    # Initialize Kokoro TTS
    print("\nInitializing Kokoro TTS engine...")
    init_start = time.time()

    try:
        from kokoro import KPipeline
    except ImportError as e:
        print(f"Error: Failed to import kokoro module: {e}")
        print("Please install kokoro: pip install kokoro")
        sys.exit(1)

    # Map language to Kokoro lang_code
    lang_map = {
        'en': 'a',
        'english': 'a',
        'zh': 'z',
        'chinese': 'z',
        'ch': 'z',
    }

    lang_code = lang_map.get(language, 'a')
    pipeline = KPipeline(lang_code=lang_code)

    init_time = time.time() - init_start
    print(f"Initialization time: {init_time:.2f}s")

    # Generate audio
    print("\nGenerating audio...")
    synthesis_start = time.time()

    # Kokoro generates audio in chunks
    audio_chunks = []
    sample_rate = 24000  # Kokoro default sample rate

    generator = pipeline(
        text,
        voice=voice,
        speed=1.0,
        split_pattern=r"\n+",
    )

    for _, (_, _, audio) in enumerate(generator):
        audio_np = audio.numpy()
        audio_chunks.append(audio_np)

    # Concatenate all chunks
    audio_data = np.concatenate(audio_chunks) if audio_chunks else np.array([])

    synthesis_time = time.time() - synthesis_start
    audio_duration = len(audio_data) / sample_rate if len(audio_data) > 0 else 0

    if len(audio_data) == 0:
        print("\n✗ Error: No audio generated!")
        sys.exit(1)

    # Calculate metrics
    chars_per_second = len(text) / synthesis_time if synthesis_time > 0 else 0
    real_time_factor = audio_duration / synthesis_time if synthesis_time > 0 else 0

    # Save audio
    output_dir = Path("tts_output")
    output_dir.mkdir(exist_ok=True)
    output_file = output_dir / f"kokoro_{language}_output.wav"

    sf.write(output_file, audio_data, sample_rate)

    # Print results
    print("\n" + "=" * 80)
    print("RESULTS")
    print("=" * 80)
    print(f"Language: {language}")
    print(f"Voice: {voice}")
    print(f"Text length: {len(text)} characters")
    print(f"Audio duration: {audio_duration:.2f} seconds")
    print(f"Synthesis time: {synthesis_time:.2f} seconds")
    print(f"Real-time factor: {real_time_factor:.2f}x {'(faster than real-time)' if real_time_factor > 1 else '(slower than real-time)'}")
    print(f"Processing speed: {chars_per_second:.1f} characters/second")
    print(f"\nAudio saved to: {output_file.absolute()}")
    print(f"File size: {output_file.stat().st_size / 1024:.1f} KB")
    print(f"Sample rate: {sample_rate} Hz")
    print("=" * 80)

    return output_file


def main():
    """Main function with argument parsing."""
    parser = argparse.ArgumentParser(
        description="Test Kokoro TTS with timing and audio file saving",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
        epilog="""
Examples:
  # Test with English (default)
  python test_tts_direct.py

  # Test with Chinese
  python test_tts_direct.py --language zh --voice af_heart

  # Test with custom text
  python test_tts_direct.py --text "Hello world!"

  # Test with different voice
  python test_tts_direct.py --voice bf_emma
        """
    )

    parser.add_argument(
        '--language',
        type=str,
        default='en',
        choices=['en', 'english', 'zh', 'chinese', 'ch', 'ja', 'japanese', 'ko', 'korean'],
        help='Language for TTS'
    )

    parser.add_argument(
        '--voice',
        type=str,
        default='af_heart',
        help='Voice to use for TTS (e.g., af_heart, bf_emma, am_adam, etc.)'
    )

    parser.add_argument(
        '--text',
        type=str,
        default=None,
        help='Custom text to synthesize (optional, uses default text if not provided)'
    )

    args = parser.parse_args()

    # Run test
    try:
        audio_file = test_tts_timing(args.language, args.voice, args.text)
        print(f"\n✓ Test completed successfully!")
        print(f"✓ Audio file: {audio_file}")
    except Exception as e:
        print(f"\n✗ Test failed: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
