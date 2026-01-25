# PrimeSpeech 音色克隆能力调研报告

> 调研日期: 2026-01-23
>
> 调研目标: 了解 dora_primespeech 是否具备音色克隆能力，以及如何使用该能力

## 1. 概述

### 1.1 结论

**dora_primespeech 具备音色克隆能力**，基于 GPT-SoVITS 技术实现。支持两种克隆方式：

| 方式 | 描述 | 适用场景 |
|-----|------|---------|
| Zero-shot 克隆 | 提供 3-10 秒参考音频即可克隆 | 快速原型、一般用途 |
| Fine-tune 训练 | 使用更多数据训练专属模型 | 高质量、生产环境 |

### 1.2 技术架构

```
┌─────────────────────────────────────────────────────────────┐
│                    dora-primespeech                         │
├─────────────────────────────────────────────────────────────┤
│  main.py          - Dora 节点入口                           │
│  config.py        - 音色配置 (VOICE_CONFIGS)                │
│  moyoyo_tts_wrapper_streaming_fix.py - TTS 包装器           │
├─────────────────────────────────────────────────────────────┤
│                    moyoyo_tts (GPT-SoVITS)                  │
├─────────────────────────────────────────────────────────────┤
│  TTS_infer_pack/TTS.py  - 核心推理引擎                      │
│  s1_train.py            - GPT 模型训练                      │
│  s2_train.py            - SoVITS 模型训练                   │
│  AR/                    - 自回归模型 (Text2Semantic)        │
│  module/                - SoVITS 声码器模块                 │
└─────────────────────────────────────────────────────────────┘
```

## 2. Zero-shot 音色克隆（推荐）

### 2.1 原理

GPT-SoVITS 的 Zero-shot 克隆通过以下方式工作：

1. **参考音频处理**: 使用 CNHuBERT 提取参考音频的语音特征
2. **语义编码**: 将参考音频的语义信息编码为 prompt_semantic
3. **条件生成**: 基于参考特征，将新文本转换为与参考音色相似的语音

### 2.2 关键要求

| 要素 | 要求 |
|-----|------|
| 参考音频时长 | 3-10 秒（推荐 5-8 秒） |
| 音频格式 | WAV, 32000Hz 采样率 |
| 音频质量 | 清晰、无噪音、无背景音乐、无混响 |
| 参考文本 | 必须准确对应参考音频内容 |

### 2.3 添加自定义音色步骤

#### 步骤 1: 准备参考音频

```bash
# 建议使用 ffmpeg 转换格式
ffmpeg -i input.mp3 -ar 32000 -ac 1 my_voice_ref.wav
```

#### 步骤 2: 编辑配置文件

文件路径: `node-hub/dora-primespeech/dora_primespeech/config.py`

```python
VOICE_CONFIGS = {
    # ... 现有音色配置 ...

    # 添加自定义音色
    "MyVoice": {
        "repository": "MoYoYoTech/tone-models",
        # 使用已有的基础模型（Zero-shot 方式）
        "gpt_weights": "GPT_weights/doubao-mixed.ckpt",
        "sovits_weights": "SoVITS_weights/doubao-mixed.pth",
        # 自定义参考音频
        "reference_audio": "ref_audios/my_voice_ref.wav",
        # 参考音频对应的文本（必须准确）
        "prompt_text": "这里是参考音频中说的话，必须准确对应音频内容。",
        # 语言设置
        "text_lang": "zh",      # 合成文本语言: zh, en, ja, ko, auto
        "prompt_lang": "zh",    # 参考音频语言
        # 可选参数
        "speed_factor": 1.0,    # 语速倍率
    },
}
```

#### 步骤 3: 放置参考音频文件

```bash
# 将参考音频放到模型目录
cp my_voice_ref.wav $PRIMESPEECH_MODEL_DIR/moyoyo/ref_audios/
```

#### 步骤 4: 使用新音色

**方式 A: 在 dataflow 配置中指定**

```yaml
# voice-chat.yml
nodes:
  - id: primespeech
    path: dora-primespeech
    inputs:
      text: llm/text
    outputs:
      - audio
    env:
      VOICE_NAME: MyVoice  # 使用自定义音色
      TEXT_LANG: zh
```

**方式 B: 运行时动态切换**

发送格式为 `VOICE:音色名|文本内容` 的消息：

```json
{"prompt": "VOICE:MyVoice|你好，这是使用自定义音色合成的语音。"}
```

## 3. Fine-tune 训练（高级）

### 3.1 何时需要 Fine-tune

- 需要更稳定、更高质量的音色复现
- 参考音频特征复杂，Zero-shot 效果不佳
- 生产环境部署，需要一致性保证

### 3.2 训练流程概述

项目中包含完整的训练代码：

```
moyoyo_tts/
├── s1_train.py          # GPT 模型训练 (Text2Semantic)
├── s2_train.py          # SoVITS 模型训练 (Vocoder)
├── AR/                  # 自回归模型定义
│   ├── data/            # 数据加载
│   └── models/          # 模型定义
└── module/              # SoVITS 模块
    ├── models.py        # SynthesizerTrn 定义
    └── data_utils.py    # 数据工具
```

### 3.3 训练步骤（概要）

1. **数据准备**
   - 收集目标音色的录音（建议 10+ 分钟）
   - 使用 ASR 工具生成文本标注
   - 切分为适当长度的音频片段

2. **特征提取**
   - 使用 CNHuBERT 提取语音特征
   - 使用 BERT 提取文本特征

3. **训练 GPT 模型 (s1_train.py)**
   ```bash
   python s1_train.py --config configs/s1.yaml
   ```

4. **训练 SoVITS 模型 (s2_train.py)**
   ```bash
   python s2_train.py --config configs/s2.yaml
   ```

5. **使用训练好的模型**
   - 将 `.ckpt` 和 `.pth` 文件放到对应目录
   - 在 config.py 中配置新音色指向训练好的模型

### 3.4 训练资源需求

| 资源 | 最低要求 | 推荐配置 |
|-----|---------|---------|
| GPU | 8GB VRAM | 16GB+ VRAM |
| 训练数据 | 3-5 分钟 | 10-30 分钟 |
| 训练时间 | 数小时 | 取决于数据量 |

## 4. 现有预置音色

### 4.1 中文音色

| 音色名 | 描述 | 特点 |
|-------|------|------|
| Doubao | 豆包 - MoYoYo 虚拟助手 | 友好、通用 |
| Luo Xiang | 罗翔 - 法学教授 | 幽默、教育风格 |
| Yang Mi | 杨幂 - 女演员 | 甜美、自然 |
| Zhou Jielun | 周杰伦 | 台湾口音 |
| Ma Yun | 马云 | 商业演讲风格 |
| Chen Yifan | 陈一凡 | 专业播报 |
| Zhao Daniu | 赵大牛 | 播客风格 |
| Ma Baoguo | 马保国 | 特色语调 |
| Shen Yi | 沈逸 | 分析评论 |
| BYS | - | 通用中文 |

### 4.2 英文音色

| 音色名 | 描述 | 特点 |
|-------|------|------|
| Maple | 女声 | 轻松、直率 |
| Cove | 男声 | 冷静、直接 |
| Ellen | 女声 | 活泼、主持风格 |
| Juniper | 女声 | 叙述风格 |
| Trump | 特朗普 | 特色口音 |

## 5. 配置参数说明

### 5.1 音色配置参数

| 参数 | 类型 | 说明 |
|-----|------|------|
| `repository` | string | HuggingFace 模型仓库 |
| `gpt_weights` | string | GPT 模型权重路径 |
| `sovits_weights` | string | SoVITS 模型权重路径 |
| `reference_audio` | string | 参考音频路径 |
| `prompt_text` | string | 参考音频对应文本 |
| `text_lang` | string | 合成文本语言 |
| `prompt_lang` | string | 参考音频语言 |
| `speed_factor` | float | 语速倍率 (默认 1.0) |

### 5.2 支持的语言代码

| 代码 | 说明 |
|-----|------|
| `auto` | 自动检测（多语种混合） |
| `zh` | 中文（中英混合） |
| `en` | 英文 |
| `ja` | 日文（日英混合） |
| `ko` | 韩文（韩英混合） |
| `yue` | 粤语（粤英混合） |
| `all_zh` | 全部按中文识别 |
| `all_ja` | 全部按日文识别 |
| `all_ko` | 全部按韩文识别 |
| `all_yue` | 全部按粤语识别 |

### 5.3 推理参数

| 参数 | 默认值 | 说明 |
|-----|-------|------|
| `TOP_K` | 5 | Top-k 采样 |
| `TOP_P` | 1.0 | Nucleus 采样 |
| `TEMPERATURE` | 1.0 | 采样温度 |
| `SPEED_FACTOR` | 1.0 | 语速倍率 |
| `BATCH_SIZE` | 100 | 批处理大小 |
| `SEED` | 233333 | 随机种子 |

## 6. 关键文件路径

| 文件 | 路径 | 说明 |
|-----|------|------|
| 节点入口 | `node-hub/dora-primespeech/dora_primespeech/main.py` | Dora 节点主程序 |
| 音色配置 | `node-hub/dora-primespeech/dora_primespeech/config.py` | VOICE_CONFIGS 定义 |
| TTS 包装器 | `node-hub/dora-primespeech/dora_primespeech/moyoyo_tts_wrapper_streaming_fix.py` | 流式 TTS 包装 |
| 核心推理 | `node-hub/dora-primespeech/dora_primespeech/moyoyo_tts/TTS_infer_pack/TTS.py` | GPT-SoVITS 推理 |
| GPT 训练 | `node-hub/dora-primespeech/dora_primespeech/moyoyo_tts/s1_train.py` | Text2Semantic 训练 |
| SoVITS 训练 | `node-hub/dora-primespeech/dora_primespeech/moyoyo_tts/s2_train.py` | 声码器训练 |
| 模型目录 | `$PRIMESPEECH_MODEL_DIR/moyoyo/` | 模型存储位置 |

## 7. 常见问题

### Q1: 克隆效果不理想怎么办？

1. **检查参考音频质量**
   - 确保无噪音、无背景音乐
   - 时长在 3-10 秒范围内
   - 语音清晰、语速适中

2. **检查 prompt_text 准确性**
   - 必须与参考音频内容完全一致
   - 包括标点符号

3. **调整推理参数**
   - 降低 `temperature` 获得更稳定输出
   - 调整 `top_k` 和 `top_p`

### Q2: 支持实时音色切换吗？

支持。发送 `VOICE:音色名|文本内容` 格式的消息即可动态切换。

### Q3: 可以混合多个参考音频吗？

支持。通过 `aux_ref_audio_paths` 参数可以提供多个辅助参考音频进行音色融合。

## 8. 参考资源

- [GPT-SoVITS 官方仓库](https://github.com/RVC-Boss/GPT-SoVITS)
- [MoYoYo 模型下载](https://huggingface.co/MoYoYoTech/tone-models)
- [Dora Framework](https://github.com/dora-rs/dora)

---

*本文档基于项目源码分析生成，如有更新请同步修改。*
