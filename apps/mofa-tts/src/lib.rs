//! MoFA TTS App - Text to Speech conversion with multiple voice options

pub mod audio_player;
pub mod dora_integration;
pub mod screen;
pub mod voice_data;
pub mod voice_selector;
pub mod log_bridge;
pub mod mofa_hero;
pub mod system_monitor;

pub use mofa_hero::{ConnectionStatus, MofaHero, MofaHeroAction};

pub use screen::TTSScreenRef;
pub use screen::TTSScreenWidgetRefExt;

use makepad_widgets::Cx;
use mofa_widgets::{AppInfo, MofaApp};

/// MoFA TTS app descriptor
pub struct MoFaTTSApp;

impl MofaApp for MoFaTTSApp {
    fn info() -> AppInfo {
        AppInfo {
            name: "TTS",
            id: "mofa-tts",
            description: "Text to Speech conversion",
            ..Default::default()
        }
    }

    fn live_design(cx: &mut Cx) {
        voice_selector::live_design(cx);
        mofa_hero::live_design(cx);
        screen::live_design(cx);
    }
}

/// Register all TTS widgets with Makepad
pub fn live_design(cx: &mut Cx) {
    MoFaTTSApp::live_design(cx);
}
