# MoFA PrimeSpeech 数据流分析

从界面点击 "Generate Speech" 按钮到 `node-hub/dora-primespeech` 节点接收数据的完整执行路径。

## 概览

```
UI Button Click → Event Handler → DoraIntegration → PromptInputBridge → Dora Node → Python TTS Node
```

## 详细执行路径

### 1. UI 层 - 按钮点击事件 (Makepad)

**文件**: `apps/mofa-primespeech/src/screen.rs`

#### 1.1 用户点击 "Generate Speech" 按钮

- **位置**: 第 361-363 行定义按钮
  ```rust
  generate_btn = <PrimaryButton> {
      text: "Generate Speech"
  }
  ```

#### 1.2 事件处理器捕获点击

- **函数**: `PrimeSpeechScreen::handle_event()` (第 845 行)
- **位置**: 第 1007-1019 行
  ```rust
  // Handle generate button
  if self
      .view
      .button(ids!(
          main_content
              .left_column
              .content_area
              .input_section
              .bottom_bar
              .generate_btn
      ))
      .clicked(actions)
  {
      self.generate_speech(cx);  // ← 调用生成语音函数
  }
  ```

### 2. 业务逻辑层 - 准备和发送数据

**文件**: `apps/mofa-primespeech/src/screen.rs`

#### 2.1 `generate_speech()` 函数 (第 1339-1425 行)

**执行步骤**:

1. **检查 Dora 连接状态** (第 1340-1348 行)

   ```rust
   let is_running = self.dora.as_ref().map(|d| d.is_running()).unwrap_or(false);
   if !is_running {
       self.add_log(cx, "[WARN] [primespeech] Bridge not connected...");
       return;
   }
   ```

2. **获取输入文本** (第 1350-1360 行)

   ```rust
   let text = self
       .view
       .text_input(ids!(
           main_content.left_column.content_area
           .input_section.input_container.text_input
       ))
       .text();
   ```

3. **验证文本非空** (第 1361-1367 行)

4. **获取选中的音色** (第 1375-1386 行)

   ```rust
   let voice_id = self
       .view
       .voice_selector(ids!(...))
       .selected_voice_id()
       .unwrap_or_else(|| "Luo Xiang".to_string());
   ```

5. **清空之前的音频缓冲** (第 1393-1395 行)

   ```rust
   self.stored_audio_samples.clear();
   self.stored_audio_sample_rate = 32000;
   ```

6. **更新UI状态为生成中** (第 1397-1398 行)

   ```rust
   self.tts_status = TTSStatus::Generating;
   self.update_player_bar(cx);
   ```

7. **编码音色到提示符** (第 1401-1403 行)

   ```rust
   // 关键: 使用 VOICE: 前缀编码音色选择
   let prompt = format!("VOICE:{}|{}", voice_id, text);
   ```

8. **通过 DoraIntegration 发送** (第 1405-1413 行)
   ```rust
   let send_result = self
       .dora
       .as_ref()
       .map(|d| d.send_prompt(&prompt))  // ← 调用 DoraIntegration
       .unwrap_or(false);
   ```

### 3. Dora 集成层 - 命令传递

**文件**: `apps/mofa-primespeech/src/dora_integration.rs`

#### 3.1 `DoraIntegration::send_prompt()` (第 116-120 行)

```rust
pub fn send_prompt(&self, message: impl Into<String>) -> bool {
    self.send_command(DoraCommand::SendPrompt {
        message: message.into(),  // ← 封装为 DoraCommand
    })
}
```

#### 3.2 `DoraIntegration::send_command()` (第 95-97 行)

```rust
pub fn send_command(&self, cmd: DoraCommand) -> bool {
    self.command_tx.try_send(cmd).is_ok()  // ← 发送到 worker 线程
}
```

### 4. Worker 线程 - 处理命令

**文件**: `apps/mofa-primespeech/src/dora_integration.rs`

#### 4.1 Worker 主循环 (第 137-308 行)

**函数**: `DoraIntegration::run_worker()`

#### 4.2 接收并处理 SendPrompt 命令 (第 221-260 行)

```rust
DoraCommand::SendPrompt { message } => {
    // 重试逻辑包装器
    let send_with_retry = |bridge: &dyn mofa_dora_bridge::DoraBridge,
                           output: &str,
                           data: mofa_dora_bridge::DoraData|
     -> Result<(), String> {
        let retries = 20;
        for attempt in 1..=retries {
            match bridge.send(output, data.clone()) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    if attempt == retries {
                        return Err(e.to_string());
                    }
                    std::thread::sleep(Duration::from_millis(150));
                }
            }
        }
        Err("retry exhausted".into())
    };

    // 查找 PromptInputBridge
    if let Some(ref disp) = dispatcher {
        if let Some(bridge) = disp
            .get_bridge("mofa-prompt-input-tts")
            .or_else(|| disp.get_bridge("mofa-prompt-input"))  // ← 获取桥接
        {
            log::info!("Sending text to TTS via bridge: {}", message);

            // 发送到桥接
            if let Err(e) = send_with_retry(
                bridge,
                "prompt",  // ← 输出端口名称
                mofa_dora_bridge::DoraData::Text(message.clone()),  // ← 数据
            ) {
                log::error!("Failed to send text: {}", e);
            }
        }
    }
}
```

### 5. Dora Bridge 层 - 数据传输

**文件**: `mofa-dora-bridge/src/widgets/prompt_input.rs`

#### 5.1 PromptInputBridge 结构 (第 1-68 行)

```rust
pub struct PromptInputBridge {
    node_id: String,
    state: Arc<RwLock<BridgeState>>,
    shared_state: Option<Arc<SharedDoraState>>,
    prompt_sender: Sender<String>,        // ← UI → Worker 通道
    prompt_receiver: Receiver<String>,    // ← Worker 接收
    // ...
}
```

#### 5.2 Bridge 的 `send()` 方法

当 DoraIntegration 调用 `bridge.send()` 时，会触发:

1. **接收 prompt** (第 124-128 行)

   ```rust
   while let Ok(prompt) = prompt_receiver.try_recv() {
       if let Err(e) = Self::send_prompt_to_dora(&mut node, &prompt) {
           warn!("Failed to send prompt: {}", e);
       }
   }
   ```

2. **发送到 Dora 节点** (位于 `prompt_input.rs` 后续部分)
   ```rust
   fn send_prompt_to_dora(node: &mut DoraNode, prompt: &str) -> BridgeResult<()> {
       // 将文本转换为 Arrow 数组
       let arrow_data = prompt.into_arrow();

       // 通过 Dora 的 "control" 输出发送
       node.send_output(
           DataId::from("control"),  // ← 对应 dataflow.yml 的输出端口
           Default::default(),
           arrow_data,
       )
       .map_err(|e| BridgeError::Unknown(e.to_string()))
   }
   ```

### 6. Dora Dataflow 配置

**文件**: `apps/mofa-primespeech/dataflow/tts.yml`

#### 6.1 节点连接配置

```yaml
nodes:
  - id: mofa-prompt-input
    path: dynamic # ← Dynamic node (由 PromptInputBridge 实现)
    outputs:
      - control # ← 输出端口

  - id: primespeech-tts
    path: dora-primespeech # ← Python TTS 节点
    inputs:
      text: mofa-prompt-input/control # ← 连接: 从 mofa-prompt-input 的 control 输出接收
    env:
      VOICE_NAME: "Ma Yun"
      # ... 其他环境变量
```

**数据流**: `mofa-prompt-input` 节点的 `control` 输出 → `primespeech-tts` 节点的 `text` 输入

### 7. Python TTS 节点 - 接收和处理

**文件**: `node-hub/dora-primespeech/dora_primespeech/main.py`

#### 7.1 节点初始化 (第 1-259 行)

主函数 `main()` 做了:

1. 初始化 Dora Node (第 88 行)

   ```python
   node = Node()
   config = PrimeSpeechConfig()
   ```

2. 加载音色配置 (第 91-104 行)

   ```python
   voice_name = config.VOICE_NAME
   if voice_name not in VOICE_CONFIGS:
       # 错误处理
   voice_config = VOICE_CONFIGS[voice_name]
   ```

3. 预初始化 TTS 引擎 (第 216-254 行)

#### 7.2 事件循环 - 接收文本 (第 260-300+ 行)

```python
for event in node:
    if event["type"] == "INPUT":
        input_id = event["id"]

        if input_id == "text":  # ← 匹配 dataflow.yml 中的 text 输入
            # 获取原始文本
            raw_text = event["value"][0].as_py()
            metadata = event.get("metadata", {})

            # 解析 VOICE: 前缀
            current_voice_name = voice_name  # 默认音色
            text = raw_text

            if raw_text.startswith("VOICE:"):
                try:
                    parts = raw_text.split("|", 1)
                    if len(parts) == 2:
                        voice_prefix = parts[0][6:].strip()  # 移除 "VOICE:"
                        text = parts[1]  # 实际文本

                        # 检查音色是否存在
                        if voice_prefix in VOICE_CONFIGS:
                            current_voice_name = voice_prefix
                            send_log(node, "INFO", f"Switching to voice: {current_voice_name}", ...)
                except Exception as e:
                    send_log(node, "WARNING", f"Failed to parse VOICE: prefix: {e}", ...)

            # 记录接收到的文本
            send_log(node, "DEBUG", f"RECEIVED text: '{text}' (voice={current_voice_name})", ...)

            # 后续会调用 TTS 引擎合成语音...
```

## 数据格式转换链

1. **UI 文本输入** (Rust String)

   ```
   "今天天气真好"
   ```

2. **添加音色前缀** (Rust String)

   ```
   "VOICE:罗翔|今天天气真好"
   ```

3. **DoraCommand 封装** (Rust enum)

   ```rust
   DoraCommand::SendPrompt {
       message: "VOICE:罗翔|今天天气真好"
   }
   ```

4. **跨线程传递** (crossbeam_channel)

   ```
   command_tx.try_send(cmd) → command_rx.try_recv()
   ```

5. **DoraData 封装** (Bridge 层)

   ```rust
   DoraData::Text("VOICE:罗翔|今天天气真好")
   ```

6. **Arrow 数组转换** (Dora 传输格式)

   ```rust
   prompt.into_arrow()  // → Arrow StringArray
   ```

7. **Dora Event** (Python 接收)

   ```python
   event = {
       "type": "INPUT",
       "id": "text",
       "value": ["VOICE:罗翔|今天天气真好"],
       "metadata": {}
   }
   ```

8. **解析和处理** (Python)
   ```python
   voice_name = "罗翔"
   text = "今天天气真好"
   ```

## 关键设计模式

### 1. 跨线程通信

- **UI 线程** → **Worker 线程**: `crossbeam_channel` (bounded, capacity=100)
- **Worker 线程** → **Dora Node**: `DoraNode::send_output()`

### 2. 重试机制

位置: `dora_integration.rs` 第 223-239 行

```rust
let send_with_retry = |bridge, output, data| -> Result<(), String> {
    let retries = 20;
    for attempt in 1..=retries {
        match bridge.send(output, data.clone()) {
            Ok(_) => return Ok(()),
            Err(e) => {
                if attempt == retries {
                    return Err(e.to_string());
                }
                std::thread::sleep(Duration::from_millis(150));  // 150ms 间隔
            }
        }
    }
    Err("retry exhausted".into())
};
```

### 3. 音色动态切换

使用特殊前缀 `VOICE:音色名|文本` 格式:

- **编码**: `apps/mofa-primespeech/src/screen.rs` 第 1403 行
- **解析**: `node-hub/dora-primespeech/dora_primespeech/main.py` 第 273-290 行

### 4. Shared State 模式

`SharedDoraState` (Arc<RwLock>) 用于:

- 存储音频数据: `shared_state.audio.drain()`
- 存储日志: `log_bridge::poll_logs()`
- 桥接状态管理: `shared_state.add_bridge()`

## 时序图

```
用户点击按钮
    ↓
handle_event() 捕获点击
    ↓
generate_speech() 准备数据
    ↓
format!("VOICE:{}|{}", voice_id, text)
    ↓
dora.send_prompt(prompt)
    ↓
command_tx.try_send(DoraCommand::SendPrompt)
    ↓
[跨线程边界]
    ↓
worker 线程 try_recv()
    ↓
dispatcher.get_bridge("mofa-prompt-input")
    ↓
bridge.send("prompt", DoraData::Text(message))
    ↓
PromptInputBridge::send_prompt_to_dora()
    ↓
node.send_output(DataId("control"), arrow_data)
    ↓
[Dora 数据流边界]
    ↓
primespeech-tts 节点接收 text 输入
    ↓
event["id"] == "text"
    ↓
解析 VOICE: 前缀
    ↓
调用 TTS 引擎合成
```

## 错误处理路径

1. **Dora 未连接**
   - 位置: `screen.rs` 第 1340-1348 行
   - 动作: 显示警告日志，提前返回

2. **文本为空**
   - 位置: `screen.rs` 第 1361-1367 行
   - 动作: 显示警告日志，提前返回

3. **发送失败**
   - 位置: `screen.rs` 第 1415-1420 行
   - 动作: 记录错误日志，更新状态为 Error

4. **桥接未找到**
   - 位置: `dora_integration.rs` 第 257-259 行
   - 动作: 记录警告日志 "mofa-prompt-input bridge not found"

5. **重试耗尽**
   - 位置: `dora_integration.rs` 第 252-254 行
   - 动作: 记录错误日志 "Failed to send text"

## 性能考虑

1. **非阻塞发送**: `try_send()` 而非 `send()` 避免 UI 冻结
2. **重试间隔**: 150ms，最多 20 次 (总共 3 秒超时)
3. **事件循环超时**: 100ms (Worker 线程的 `recv_timeout`)
4. **音频缓冲**: 存储在 `stored_audio_samples` 向量中，避免实时播放导致的抖动

## 相关文件清单

### UI 层

- `apps/mofa-primespeech/src/screen.rs` - 主界面和事件处理
- `apps/mofa-primespeech/src/lib.rs` - 应用入口

### 集成层

- `apps/mofa-primespeech/src/dora_integration.rs` - Dora 集成管理
- `apps/mofa-primespeech/src/audio_player.rs` - 音频播放

### 桥接层

- `mofa-dora-bridge/src/widgets/prompt_input.rs` - PromptInputBridge 实现
- `mofa-dora-bridge/src/dispatcher.rs` - 桥接调度器
- `mofa-dora-bridge/src/shared_state.rs` - 共享状态

### Dataflow 配置

- `apps/mofa-primespeech/dataflow/tts.yml` - Dora dataflow 定义

### Python 节点

- `node-hub/dora-primespeech/dora_primespeech/main.py` - TTS 节点主逻辑
- `node-hub/dora-primespeech/dora_primespeech/config.py` - 配置管理
- `node-hub/dora-primespeech/dora_primespeech/moyoyo_tts_wrapper_streaming_fix.py` - TTS 引擎封装

## 总结

整个数据流遵循清晰的分层架构:

1. **UI 层** (Makepad): 事件捕获和用户交互
2. **业务逻辑层** (Screen): 数据准备和状态管理
3. **集成层** (DoraIntegration): 命令封装和异步处理
4. **桥接层** (PromptInputBridge): 协议转换和 Dora 通信
5. **Dataflow 层** (YAML): 节点连接配置
6. **处理层** (Python): TTS 合成和音频生成

每层职责明确，通过标准接口通信，实现了良好的解耦和可维护性。
