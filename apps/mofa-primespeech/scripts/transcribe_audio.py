#!/usr/bin/env python3
"""
Standalone audio transcription script for voice cloning.
Uses FunASR for Chinese, Whisper for other languages.

Usage:
    python transcribe_audio.py <audio_file> [--language zh|en|auto]

Output:
    JSON: {"text": "transcription", "language": "detected_language"}

Dependencies:
    - soundfile: for audio loading
    - librosa: for resampling (optional, falls back to simple resampling)
    - funasr: for Chinese ASR (optional)
    - openai-whisper: for multi-language ASR (optional)

Install via:
    pip install soundfile librosa funasr openai-whisper
"""

import sys
import os
import json
import argparse

# Check for numpy first
try:
    import numpy as np
except ImportError:
    print(json.dumps({'error': 'numpy not installed. Run: pip install numpy'}))
    sys.exit(1)

# Add node-hub to path for reusing ASR engines
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..', '..', 'node-hub', 'dora-asr'))


def load_audio(audio_path: str, target_sr: int = 16000) -> np.ndarray:
    """Load audio file and convert to 16kHz mono float32."""
    try:
        import soundfile as sf
    except ImportError:
        raise ImportError('soundfile not installed. Run: pip install soundfile')

    # Load audio
    audio, sr = sf.read(audio_path, dtype='float32')

    # Convert to mono if stereo
    if len(audio.shape) > 1:
        audio = audio.mean(axis=1)

    # Resample if needed
    if sr != target_sr:
        try:
            import librosa
            audio = librosa.resample(audio, orig_sr=sr, target_sr=target_sr)
        except ImportError:
            # Fallback: simple linear interpolation resampling
            ratio = target_sr / sr
            new_length = int(len(audio) * ratio)
            indices = np.linspace(0, len(audio) - 1, new_length)
            audio = np.interp(indices, np.arange(len(audio)), audio).astype(np.float32)

    return audio


def transcribe_with_funasr(audio: np.ndarray, language: str = 'zh') -> dict:
    """Transcribe using FunASR (best for Chinese)."""
    try:
        from funasr import AutoModel
    except ImportError:
        return {'text': '', 'language': language, 'error': 'FunASR not installed. Run: pip install funasr'}

    try:
        # Get models directory
        models_dir = os.environ.get('MOFA_MODELS_DIR',
            os.path.join(os.path.dirname(__file__), '..', '..', '..', 'models'))

        # Initialize model
        model = AutoModel(
            model="paraformer-zh",
            model_revision="v2.0.4",
            vad_model="fsmn-vad",
            vad_model_revision="v2.0.4",
            punc_model="ct-punc-c",
            punc_model_revision="v2.0.4",
        )

        # Transcribe
        result = model.generate(input=audio, batch_size_s=300)

        if result and len(result) > 0:
            text = result[0].get('text', '')
            return {'text': text, 'language': language}

        return {'text': '', 'language': language}

    except Exception as e:
        print(f"FunASR error: {e}", file=sys.stderr)
        return {'text': '', 'language': language, 'error': str(e)}


def transcribe_with_whisper(audio: np.ndarray, language: str = 'auto') -> dict:
    """Transcribe using Whisper (multi-language support)."""
    try:
        import whisper
    except ImportError:
        return {'text': '', 'language': language, 'error': 'Whisper not installed. Run: pip install openai-whisper'}

    try:
        # Load model (use base for speed, medium for accuracy)
        model_name = os.environ.get('WHISPER_MODEL', 'base')
        model = whisper.load_model(model_name)

        # Transcribe
        options = {}
        if language != 'auto':
            options['language'] = language

        result = model.transcribe(audio, **options)

        detected_lang = result.get('language', language)
        text = result.get('text', '').strip()

        return {'text': text, 'language': detected_lang}

    except Exception as e:
        print(f"Whisper error: {e}", file=sys.stderr)
        return {'text': '', 'language': language, 'error': str(e)}


def transcribe(audio_path: str, language: str = 'auto') -> dict:
    """
    Transcribe audio file.

    Args:
        audio_path: Path to audio file
        language: Language hint ('zh', 'en', 'auto')

    Returns:
        Dict with 'text' and 'language' keys
    """
    # Load audio
    try:
        audio = load_audio(audio_path)
    except Exception as e:
        return {'error': f'Failed to load audio: {e}'}

    errors = []

    # Select engine based on language
    if language == 'zh':
        # Use FunASR for Chinese
        result = transcribe_with_funasr(audio, language)
        if not result.get('error') and result.get('text'):
            return result
        if result.get('error'):
            errors.append(f"FunASR: {result['error']}")

        # Fallback to Whisper
        result = transcribe_with_whisper(audio, language)
        if not result.get('error'):
            return result
        errors.append(f"Whisper: {result['error']}")

    elif language == 'en':
        # Use Whisper for English
        result = transcribe_with_whisper(audio, language)
        if not result.get('error'):
            return result
        errors.append(f"Whisper: {result['error']}")

    else:
        # Auto-detect: try Whisper first for detection
        result = transcribe_with_whisper(audio, 'auto')

        if result.get('error'):
            errors.append(f"Whisper: {result['error']}")
        else:
            detected_lang = result.get('language', 'en')

            # If Chinese detected and FunASR available, re-transcribe with FunASR
            if detected_lang in ['zh', 'chinese', 'mandarin']:
                funasr_result = transcribe_with_funasr(audio, 'zh')
                if not funasr_result.get('error') and funasr_result.get('text'):
                    return funasr_result

            return result

    # If all engines failed, return error
    if errors:
        return {'error': 'No ASR engine available. ' + '; '.join(errors)}

    return {'error': 'No ASR engine available'}


def main():
    parser = argparse.ArgumentParser(description='Transcribe audio for voice cloning')
    parser.add_argument('audio_file', help='Path to audio file')
    parser.add_argument('--language', '-l', default='auto',
                        choices=['zh', 'en', 'auto'],
                        help='Language hint (default: auto)')
    parser.add_argument('--output', '-o', default=None,
                        help='Output file (default: stdout)')

    args = parser.parse_args()

    # Check file exists
    if not os.path.exists(args.audio_file):
        print(json.dumps({'error': f'File not found: {args.audio_file}'}))
        sys.exit(1)

    # Transcribe
    result = transcribe(args.audio_file, args.language)

    # Output
    output_json = json.dumps(result, ensure_ascii=False)

    if args.output:
        with open(args.output, 'w', encoding='utf-8') as f:
            f.write(output_json)
    else:
        print(output_json)


if __name__ == '__main__':
    main()
