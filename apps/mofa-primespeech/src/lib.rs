//! MoFA PrimeSpeech App - Text to Speech using GPT-SoVITS with voice cloning

// Local modules
pub mod audio_player; // Keep local: simplified TTS-specific version
pub mod dora_integration;
pub mod screen;
pub mod voice_clone_modal;
pub mod voice_data;
pub mod voice_persistence;
pub mod voice_selector;

// Re-export shared components from mofa-ui
pub use mofa_ui::log_bridge;
pub use mofa_ui::system_monitor;
pub use mofa_ui::widgets::mofa_hero::{self, ConnectionStatus, MofaHero, MofaHeroAction};

pub use screen::PrimeSpeechScreenRef;
pub use screen::PrimeSpeechScreenWidgetRefExt;

use makepad_widgets::Cx;
use mofa_widgets::{AppInfo, MofaApp};

/// MoFA PrimeSpeech app descriptor
pub struct MoFaPrimeSpeechApp;

impl MofaApp for MoFaPrimeSpeechApp {
    fn info() -> AppInfo {
        AppInfo {
            name: "PrimeSpeech",
            id: "mofa-primespeech",
            description: "GPT-SoVITS Text to Speech with voice cloning",
            ..Default::default()
        }
    }

    fn live_design(cx: &mut Cx) {
        // Register shared components
        mofa_ui::widgets::mofa_hero::live_design(cx);

        // Register local components
        voice_selector::live_design(cx);
        voice_clone_modal::live_design(cx);
        screen::live_design(cx);
    }
}

/// Register all PrimeSpeech widgets with Makepad
pub fn live_design(cx: &mut Cx) {
    MoFaPrimeSpeechApp::live_design(cx);
}
