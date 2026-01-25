//! MoFA PrimeSpeech App - Text to Speech using GPT-SoVITS with voice cloning

pub mod audio_player;
pub mod dora_integration;
pub mod log_bridge;
pub mod mofa_hero;
pub mod screen;
pub mod system_monitor;
pub mod voice_clone_modal;
pub mod voice_data;
pub mod voice_persistence;
pub mod voice_selector;

pub use mofa_hero::{ConnectionStatus, MofaHero, MofaHeroAction};

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
        voice_selector::live_design(cx);
        voice_clone_modal::live_design(cx);
        mofa_hero::live_design(cx);
        screen::live_design(cx);
    }
}

/// Register all PrimeSpeech widgets with Makepad
pub fn live_design(cx: &mut Cx) {
    MoFaPrimeSpeechApp::live_design(cx);
}
