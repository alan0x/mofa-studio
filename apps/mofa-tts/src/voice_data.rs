//! Voice data definitions for TTS

use serde::{Deserialize, Serialize};

/// Voice information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Voice {
    /// Unique voice ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Voice style/category
    pub category: VoiceCategory,
    /// Preview audio file path (optional)
    pub preview_audio: Option<String>,
}

/// Voice category
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum VoiceCategory {
    Male,
    Female,
    Character,
}

impl VoiceCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            VoiceCategory::Male => "Male",
            VoiceCategory::Female => "Female",
            VoiceCategory::Character => "Character",
        }
    }
}

/// Get built-in voices
pub fn get_builtin_voices() -> Vec<Voice> {
    vec![
        Voice {
            id: "zm_yunjian".to_string(), // Mapped to Kokoro Male
            name: "Luo Xiang".to_string(),
            description: "A wise and approachable legal expert with clear articulation".to_string(),
            category: VoiceCategory::Male,
            preview_audio: None,
        },
        Voice {
            id: "zf_xiaoxiao".to_string(), // Mapped to Kokoro Female
            name: "Yang Mi".to_string(),
            description: "A sweet and expressive female voice with natural charm".to_string(),
            category: VoiceCategory::Female,
            preview_audio: None,
        },
        Voice {
            id: "zm_yunxi".to_string(), // Another Male
            name: "Zhao Dan Niu".to_string(),
            description: "A confident male broadcaster with professional tone".to_string(),
            category: VoiceCategory::Male,
            preview_audio: None,
        },
        Voice {
            id: "zm_jian".to_string(),
            name: "Chen Yi Fan".to_string(),
            description: "A young and energetic voice for casual conversations".to_string(),
            category: VoiceCategory::Male,
            preview_audio: None,
        },
        Voice {
            id: "zf_xiaoni".to_string(),
            name: "Storyteller".to_string(),
            description: "A warm narrative voice perfect for audiobooks and stories".to_string(),
            category: VoiceCategory::Character,
            preview_audio: None,
        },
        Voice {
            id: "zf_xiaoyi".to_string(),
            name: "News Anchor".to_string(),
            description: "A professional and authoritative broadcasting voice".to_string(),
            category: VoiceCategory::Female,
            preview_audio: None,
        },
    ]
}

/// TTS generation status
#[derive(Clone, Debug, PartialEq)]
pub enum TTSStatus {
    Idle,
    Generating,
    Ready,
    Playing,
    Error(String),
}

impl Default for TTSStatus {
    fn default() -> Self {
        TTSStatus::Idle
    }
}
