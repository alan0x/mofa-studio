//! MoFA Friend App - A social interaction application for Makepad

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

pub use screen::FriendScreenRef;
pub use screen::FriendScreenWidgetRefExt;

use makepad_widgets::Cx;
use mofa_widgets::{AppInfo, MofaApp};

/// MoFA Friend app descriptor
pub struct MoFaFriendApp;

impl MofaApp for MoFaFriendApp {
    fn info() -> AppInfo {
        AppInfo {
            name: "Friend",
            id: "mofa-friend",
            description: "Connect and interact with friends",
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

/// Register all Friend widgets with Makepad
pub fn live_design(cx: &mut Cx) {
    MoFaFriendApp::live_design(cx);
}
