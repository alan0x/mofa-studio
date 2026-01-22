# MoFA PrimeSpeech - Start Button Code Execution Path

> 详细记录从点击 Start 按钮到 Dora 数据流启动的完整代码执行路径

## 执行流程概览

```
用户点击 "Start MoFA" 按钮
    ↓
MofaHero::handle_event()        [mofa_hero.rs:542-554]
    ↓
发送 MofaHeroAction::StartClicked
    ↓
PrimeSpeechScreen::handle_event() [screen.rs:945-946]
    ↓
PrimeSpeechScreen::start_dora()   [screen.rs:1261-1308]
    ↓
DoraIntegration::start_dataflow() [dora_integration.rs:103-108]
    ↓
DoraIntegration::run_worker()     [dora_integration.rs:137-301]
    ↓
DataflowController::new() + DynamicNodeDispatcher::start()
    ↓
Dora 节点初始化并连接
```

---

## 详细步骤分解

### 1. UI 按钮点击 - MofaHero Widget

**文件**: `apps/mofa-primespeech/src/mofa_hero.rs`

**函数**: `MofaHero::handle_event()` (Lines 521-555)

```rust
// Lines 542-554: 检测 start button view 的点击
let start_view = self.view.view(ids!(action_section.start_view));
let stop_view = self.view.view(ids!(action_section.stop_view));

match event.hits(cx, start_view.area()) {
    Hit::FingerUp(_) => {
        cx.widget_action(self.widget_uid(), &scope.path, MofaHeroAction::StartClicked);
    }
    _ => {}
}
```

**发送的 Action**: `MofaHeroAction::StartClicked` (Line 544)

**UI 组件位置**:
- Start button view: `live_design!` at Lines 303-327
- 标签: "Start MoFA"
- 图标: 绿色 SVG 播放图标

---

### 2. Action 路由 - PrimeSpeechScreen

**文件**: `apps/mofa-primespeech/src/screen.rs`

**函数**: `PrimeSpeechScreen::handle_event()` (Lines 844-1095)

**关键代码** (Lines 937-952):

```rust
// 处理 MofaHero Actions (Start/Stop)
let actions = match event {
    Event::Actions(actions) => actions.as_slice(),
    _ => &[],
};

for action in actions {
    match action.as_widget_action().cast() {
        MofaHeroAction::StartClicked => {
            self.start_dora(cx);  // ← 关键函数调用
        }
        MofaHeroAction::StopClicked => {
            self.stop_dora(cx);
        }
        MofaHeroAction::None => {}
    }
    // ... 更多 action 处理
}
```

---

### 3. Dora 启动 - PrimeSpeechScreen

**文件**: `apps/mofa-primespeech/src/screen.rs`

**函数**: `PrimeSpeechScreen::start_dora()` (Lines 1261-1308)

**关键逻辑**:

```rust
fn start_dora(&mut self, cx: &mut Cx) {
    // Line 1263: 检查 dora 是否已在运行
    let should_start = self.dora.as_ref().map(|d| !d.is_running()).unwrap_or(false);
    if !should_start { return; }

    // Line 1269: 解析 dataflow YAML 路径
    let dataflow_path = PathBuf::from("apps/mofa-primespeech/dataflow/tts.yml");

    // Line 1270: 验证文件存在
    if !dataflow_path.exists() {
        // 错误处理...
        return;
    }

    // Line 1286: 发送命令到 DoraIntegration worker 线程
    if let Some(dora) = &mut self.dora {
        dora.start_dataflow(dataflow_path);  // ← 触发异步启动
    }

    // Lines 1290-1307: 更新 UI 状态 "Connecting" → "Connected"
    self.view.mofa_hero(ids!(main_content.left_column.hero))
        .set_running(cx, true);
    self.view.mofa_hero(ids!(main_content.left_column.hero))
        .set_connection_status(cx, ConnectionStatus::Connecting);
    // ... 状态更新
}
```

**Dataflow YAML 路径**: `apps/mofa-primespeech/dataflow/tts.yml`

---

### 4. 异步命令路由 - DoraIntegration

**文件**: `apps/mofa-primespeech/src/dora_integration.rs`

**函数**: `DoraIntegration::start_dataflow()` (Lines 103-108)

```rust
pub fn start_dataflow(&self, dataflow_path: impl Into<PathBuf>) -> bool {
    self.send_command(DoraCommand::StartDataflow {
        dataflow_path: dataflow_path.into(),
        env_vars: std::collections::HashMap::new(),
    })
}
```

**底层实现**: `DoraIntegration::send_command()` (Lines 98-100)

```rust
pub fn send_command(&self, cmd: DoraCommand) -> bool {
    self.command_tx.try_send(cmd).is_ok()  // ← 发送到 Worker 线程
}
```

**Channel**: `crossbeam_channel::bounded(100)` (Line 60)

**Command 类型**:
```rust
pub enum DoraCommand {
    StartDataflow {
        dataflow_path: PathBuf,
        env_vars: std::collections::HashMap<String, String>,
    },
    StopDataflow,
    SendPrompt { message: String },
}
```

---

### 5. Worker 线程处理 - DoraIntegration

**文件**: `apps/mofa-primespeech/src/dora_integration.rs`

**函数**: `DoraIntegration::run_worker()` (Lines 137-301) - 在独立线程中运行

**关键执行步骤**:

**Step A** - 命令接收 (Lines 160-205):

```rust
while let Ok(cmd) = command_rx.try_recv() {
    match cmd {
        DoraCommand::StartDataflow {
            dataflow_path,
            env_vars,
        } => {
            log::info!("Starting dataflow: {:?}", dataflow_path);

            // Step 1: 应用环境变量
            for (key, value) in &env_vars {
                std::env::set_var(key, value);
            }

            // Step 2: 创建 DataflowController
            match DataflowController::new(&dataflow_path) {
                Ok(mut controller) => {
                    controller.set_envs(env_vars.clone());

                    // Step 3: 创建 DynamicNodeDispatcher
                    let mut disp = DynamicNodeDispatcher::with_shared_state(
                        controller,
                        Arc::clone(&shared_state_for_dispatcher),
                    );

                    // Step 4: 启动 dispatcher (启动 dora + bridges)
                    match disp.start() {
                        Ok(dataflow_id) => {
                            log::info!("Dataflow started: {}", dataflow_id);
                            running.store(true, Ordering::Release);
                            dataflow_start_time = Some(std::time::Instant::now());
                            let _ = event_tx.send(DoraEvent::DataflowStarted { dataflow_id });
                            dispatcher = Some(disp);
                        }
                        Err(e) => { /* 错误处理 */ }
                    }
                }
                Err(e) => { /* 错误处理 */ }
            }
        }
    }
}
```

---

### 6. Dataflow Controller 初始化

**文件**: `mofa-dora-bridge/src/controller.rs`

**函数**: `DataflowController::new()` (Lines 68-87)

```rust
pub fn new(dataflow_path: impl AsRef<Path>) -> BridgeResult<Self> {
    let path = dataflow_path.as_ref()
        .canonicalize()
        .unwrap_or_else(|_| dataflow_path.as_ref().to_path_buf());

    // 解析 YAML dataflow 定义
    let parsed = DataflowParser::parse(&path)?;

    Ok(Self {
        dataflow_path: path,
        parsed: Some(parsed),
        state: Arc::new(RwLock::new(DataflowState::Stopped)),
        env_vars: HashMap::new(),
        daemon_process: None,
    })
}
```

**Dataflow YAML 解析**: 从 `apps/mofa-primespeech/dataflow/tts.yml` 加载节点定义

---

### 7. Dynamic Node Dispatcher - Start

**文件**: `mofa-dora-bridge/src/dispatcher.rs`

**函数**: `DynamicNodeDispatcher::start()` (Lines 220-270)

```rust
pub fn start(&mut self) -> BridgeResult<String> {
    // Step 1: 通过 controller 启动 dataflow
    let dataflow_id = {
        let mut controller = self.controller.write();
        controller.start()?  // ← 调用 dora start
    };

    // Step 2: 等待 dataflow 初始化 (dora 注册动态节点)
    info!("Waiting for dataflow to initialize...");
    std::thread::sleep(std::time::Duration::from_secs(2));  // ← 等待期

    // Step 3: 为动态节点创建 bridges
    if self.bridges.is_empty() {
        self.create_bridges()?;
    }

    // Step 4: 连接 bridges 到 dataflow (最多 15 次尝试)
    info!("Connecting {} bridges to dora...", self.bridges.len());
    const MAX_CONNECT_ATTEMPTS: usize = 15;
    let connect_retry_delay = std::time::Duration::from_secs(2);

    let mut last_err: Option<BridgeError> = None;
    for attempt in 1..=MAX_CONNECT_ATTEMPTS {
        match self.connect_all() {
            Ok(()) => {
                info!("All bridges connected after {} attempt(s)", attempt);
                last_err = None;
                break;
            }
            Err(e) => {
                warn!("Bridge connection attempt {} failed: {}. Retrying...", attempt, e);
                last_err = Some(e);
                std::thread::sleep(connect_retry_delay);
            }
        }
    }

    if let Some(err) = last_err {
        return Err(err);
    }

    Ok(dataflow_id)
}
```

---

### 8. Dora Dataflow YAML 配置

**文件**: `apps/mofa-primespeech/dataflow/tts.yml`

```yaml
nodes:
  - id: mofa-prompt-input
    path: dynamic
    outputs:
      - control

  - id: primespeech-tts
    path: dora-primespeech
    inputs:
      text: mofa-prompt-input/control
    outputs:
      - audio
      - status
      - segment_complete
      - log
    env:
      TRANSFORMERS_OFFLINE: "1"
      HF_HUB_OFFLINE: "1"
      VOICE_NAME: "Ma Yun"
      PRIMESPEECH_MODEL_DIR: $HOME/.dora/models/primespeech
      TEXT_LANG: zh
      PROMPT_LANG: zh
      TOP_K: 5
      TOP_P: 1.0
      TEMPERATURE: 1.0
      SPEED_FACTOR: 1.1
      USE_GPU: false
      NUM_THREADS: 4

  - id: mofa-audio-player
    path: dynamic
    inputs:
      audio: primespeech-tts/audio
    outputs:
      - buffer_status
```

**关键节点**:

| 节点 ID | 类型 | 功能 |
|---------|------|------|
| `mofa-prompt-input` | Dynamic (UI Bridge) | UI 发送文本到这里 |
| `primespeech-tts` | Python Node | TTS 处理节点 |
| `mofa-audio-player` | Dynamic (UI Bridge) | 音频输出节点 |

---

## Bridge 创建和连接流程

**文件**: `mofa-dora-bridge/src/dispatcher.rs`

**函数**: `DynamicNodeDispatcher::create_bridges()` (Lines 89-134)

为 `mofa-prompt-input` 创建 PromptInputBridge 连接到 Dora:

```rust
pub fn create_bridges(&mut self) -> BridgeResult<()> {
    let mofa_nodes = self.discover_mofa_nodes();
    let shared_state = Some(self.shared_state.clone());

    for node_spec in mofa_nodes {
        let bridge: Box<dyn DoraBridge> = match node_spec.node_type {
            MofaNodeType::PromptInput => Box::new(
                PromptInputBridge::with_shared_state(
                    &node_spec.id,
                    shared_state.clone(),
                )
            ),
            MofaNodeType::AudioPlayer => Box::new(
                AudioPlayerBridge::with_shared_state(
                    &node_spec.id,
                    shared_state.clone(),
                )
            ),
            // ... 其他 bridge 类型
        };
        self.bridges.insert(node_spec.id, bridge);
    }
    Ok(())
}
```

### PromptInputBridge

**文件**: `mofa-dora-bridge/src/widgets/prompt_input.rs`

- **初始化**: `PromptInputBridge::with_shared_state()` (Lines 55-70)
- **连接**: 在 worker 线程中调用 `DoraNode::init_from_node_id()` (Line 99)
- **功能**: 发送 prompts/text 到 dora 节点，接收响应

---

## 消息流: 从文本到音频

Dataflow 启动后，用户生成语音的流程:

**文件**: `apps/mofa-primespeech/src/screen.rs`

**函数**: `PrimeSpeechScreen::generate_speech()` (Lines 1337-1424)

```rust
fn generate_speech(&mut self, cx: &mut Cx) {
    // 从 UI 获取文本
    let text = self.view.text_input(ids!(...)).text();

    // 获取选择的语音
    let voice_id = self.view.voice_selector(ids!(...))
        .selected_voice_id()
        .unwrap_or_else(|| "Luo Xiang".to_string());

    // 编码格式: "VOICE:voice_name|text"
    let prompt = format!("VOICE:{}|{}", voice_id, text);

    // 通过 PromptInputBridge 发送到 dora
    let send_result = self.dora.as_ref()
        .map(|d| d.send_prompt(&prompt))
        .unwrap_or(false);
}
```

**发送路径**:

```
PrimeSpeechScreen::generate_speech()
  ↓ DoraIntegration::send_prompt()
  ↓ DoraIntegration::send_command(DoraCommand::SendPrompt)
  ↓ Worker 线程处理: get_bridge("mofa-prompt-input")
  ↓ PromptInputBridge::send_prompt()
  ↓ 发送到 "control" output
  ↓ primespeech-tts 节点在 "text" input 接收
  ↓ 生成音频
  ↓ 发送到 "audio" output
  ↓ mofa-audio-player 节点 + SharedDoraState.audio
  ↓ UI 轮询并接收音频样本
```

---

## Shared State 通信

**文件**: `mofa-dora-bridge/src/shared_state.rs`

**结构**: `SharedDoraState` (被 Dora 节点和 UI 共同使用)

```rust
pub struct SharedDoraState {
    pub chat: DirtyVec<ChatMessage>,
    pub logs: DirtyVec<LogEntry>,
    pub audio: AudioState,
    pub error: DirtyValue<Option<String>>,
}
```

**UI 轮询** - `PrimeSpeechScreen::handle_event()` (Lines 881-912):

```rust
if self.update_timer.is_event(event).is_some() {
    // 从 shared state 轮询音频
    if let Some(dora) = &self.dora {
        if dora.is_running() {
            let shared = dora.shared_dora_state();
            let chunks = shared.audio.drain();  // ← 获取音频
            if !chunks.is_empty() {
                for audio in chunks {
                    self.stored_audio_samples.extend(&audio.samples);
                    self.stored_audio_sample_rate = audio.sample_rate;
                }
                // 转换到 Ready 状态
                if self.tts_status == TTSStatus::Generating {
                    self.tts_status = TTSStatus::Ready;
                    self.update_player_bar(cx);
                }
            }
        }
    }
    // 轮询日志
    let logs = log_bridge::poll_logs();
    if !logs.is_empty() {
        for log_msg in logs {
            self.log_entries.push(log_msg.format());
        }
        self.update_log_display(cx);
    }
}
```

---

## 关键函数调用链总结

```
MofaHero::handle_event()
  → 发送 MofaHeroAction::StartClicked

PrimeSpeechScreen::handle_event()
  → 检测 action
  → 调用 start_dora()

PrimeSpeechScreen::start_dora()
  → 检查 DoraIntegration::is_running()
  → 解析 dataflow 路径: "apps/mofa-primespeech/dataflow/tts.yml"
  → DoraIntegration::start_dataflow(path)

DoraIntegration::start_dataflow()
  → send_command(DoraCommand::StartDataflow {...})
  → command_tx.try_send() 到 worker 线程

DoraIntegration::run_worker() [worker 线程]
  → 接收 DoraCommand::StartDataflow
  → DataflowController::new(&dataflow_path)
    → DataflowParser::parse()
  → DynamicNodeDispatcher::with_shared_state()
  → DynamicNodeDispatcher::start()
    → controller.start() [启动 dora 进程]
    → sleep 2s
    → create_bridges()
    → connect_all() [15 次尝试, 2s 重试间隔]

UI 通过定时器轮询 (100ms):
  → PrimeSpeechScreen::handle_event()
  → 检查 dora.is_running()
  → shared_dora_state().audio.drain()
  → 显示音频
```

---

## 涉及的文件

| 功能 | 文件路径 |
|------|----------|
| **UI 按钮** | `apps/mofa-primespeech/src/mofa_hero.rs` |
| **Action 处理** | `apps/mofa-primespeech/src/screen.rs` |
| **Dora 集成** | `apps/mofa-primespeech/src/dora_integration.rs` |
| **Dataflow YAML** | `apps/mofa-primespeech/dataflow/tts.yml` |
| **Controller** | `mofa-dora-bridge/src/controller.rs` |
| **Dispatcher** | `mofa-dora-bridge/src/dispatcher.rs` |
| **Prompt Bridge** | `mofa-dora-bridge/src/widgets/prompt_input.rs` |
| **Audio Bridge** | `mofa-dora-bridge/src/widgets/audio_player.rs` |
| **Shared State** | `mofa-dora-bridge/src/shared_state.rs` |

---

## 架构特点

这个执行路径展示了清晰的分层架构:

1. **UI 层**: Makepad widgets 处理用户交互
2. **集成层**: `DoraIntegration` 管理异步命令和事件
3. **桥接层**: `DynamicNodeDispatcher` + Bridges 连接 UI 和 Dora
4. **数据流层**: Dora dataflow 处理 AI 管道

线程安全通过 `SharedDoraState` 和 channel-based 通信实现。
