# PrimeSpeech TTS Validation

This validation suite tests the performance and functionality of the dora-primespeech Text-to-Speech node with Chinese text input.

## Overview

The validation measures:
- TTS synthesis time for long Chinese text (160 characters)
- Real-time factor (RTF) - ratio of audio duration to processing time
- Character processing speed
- Audio quality and file size

## Test Results

### Performance Metrics (CPU-based)

| Metric | Value |
|--------|-------|
| Text Length | 160 Chinese characters |
| Audio Duration | 31.97 seconds |
| Synthesis Time | 41.84 seconds |
| Real-time Factor | 0.76x (slower than real-time) |
| Processing Speed | 3.8 characters/second |
| Audio File Size | 2 MB |
| Audio Format | WAV 16-bit PCM, mono, 32kHz |

### Hardware Configuration

- **Device**: CPU (no GPU acceleration)
- **Model**: MoYoYo TTS with Doubao voice
- **Framework**: PyTorch

## Directory Structure

```
primespeech-validation/
├── README.md                          # This documentation
├── test_tts_direct.py                 # Standalone TTS test (no dataflow)
├── dataflow-static.yml                # Static node dataflow configuration
├── simple_text_sender_static.py       # Static text sender node
├── audio_recorder_static.py           # Static audio recorder node
├── tts_output/                        # Generated audio files
│   ├── chinese_tts_output.wav        # Sample output (32 seconds)
│   └── timing_results.json           # Performance metrics
└── logs/                              # Test execution logs
    └── static_test.log                # Dataflow execution log
```

## Quick Start

### 1. Direct TTS Test (Recommended)

Test TTS performance without Dora dataflow:

```bash
cd /home/user/dora/examples/setup-new-chatbot/primespeech-validation
python test_tts_direct.py
```

This will:
- Initialize the TTS engine
- Generate audio from 160 Chinese characters
- Save audio to `tts_output/chinese_tts_output.wav`
- Display timing metrics

### 2. Dataflow Test

Test with full Dora dataflow pipeline:

```bash
# Kill any existing Dora instances
dora destroy

# Start the dataflow, ctrl C to break the dataflow because it will not exit by iteslef
dora up
dora start dataflow-static.yml
```

This runs:
- Text sender → Text segmenter → PrimeSpeech TTS → Audio recorder
- Saves audio to `tts_output/test_output.wav`

## Test Text

The benchmark uses a 160-character Chinese text about China's modernization strategy:

```
我们说中国式现代化是百年大战略，这又分为三个阶段。第一个阶段，我们先用30年时间建成了独立完整的工业体系和国民经济体系；再用40年，到2021年，全面建成了小康社会。我们现在正处于第三个阶段，这又被分成上下两篇：上半篇是到2035年基本实现社会主义现代化；下半篇是到本世纪中叶，也就是2050年，建成社会主义现代化强国。
```

## Configuration

### Environment Variables

```bash
# Model directory
export PRIMESPEECH_MODEL_DIR=$HOME/.dora/models/primespeech

# TTS settings
export VOICE_NAME=Doubao
export TEXT_LANG=zh
export USE_GPU=false
export DEVICE=cpu
export SPEED_FACTOR=1.0
```

### Text Segmenter Settings

```yaml
MAX_SEGMENT_LENGTH: 100
MIN_SEGMENT_LENGTH: 20
SEGMENT_MODE: sentence
PUNCTUATION_MARKS: "。！？.!?；;"
```

## Performance Analysis

### Real-time Factor (RTF)

- **RTF = 0.76x**: The system generates audio slower than real-time on CPU
- For real-time applications, RTF should be > 1.0
- GPU acceleration can improve RTF to > 2.0x

### Optimization Opportunities

1. **Enable GPU**: Set `USE_GPU=true` and `DEVICE=cuda` for ~3x speedup
2. **Batch Processing**: Process multiple segments in parallel
3. **Model Quantization**: Use INT8 models for faster inference
4. **Streaming Mode**: Enable streaming for lower latency

## Troubleshooting

### Common Issues

1. **Missing G2PW Model**
   ```bash
   cd /home/user/dora/examples/model-manager
   python download_models.py --download g2pw
   ```

2. **Memory Issues**
   - Reduce `MAX_SEGMENT_LENGTH` in dataflow
   - Enable streaming mode
   - Use smaller batch sizes

3. **Audio Quality**
   - Adjust `SPEED_FACTOR` (0.8-1.2 range)
   - Try different voices in the model

## Dependencies

- Python 3.12
- dora-primespeech
- dora-text-segmenter
- PyTorch (CPU or CUDA)
- NumPy 1.26.4
- soundfile

## License

See main Dora project license.