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
        // Chinese voices
        Voice {
            id: "zm_yunjian".to_string(),
            name: "云剑 (Yunjian)".to_string(),
            description: "Chinese male - wise and clear articulation".to_string(),
            category: VoiceCategory::Male,
            preview_audio: None,
        },
        Voice {
            id: "zf_xiaoxiao".to_string(),
            name: "小小 (Xiaoxiao)".to_string(),
            description: "Chinese female - sweet and expressive".to_string(),
            category: VoiceCategory::Female,
            preview_audio: None,
        },
        Voice {
            id: "zm_yunxi".to_string(),
            name: "云希 (Yunxi)".to_string(),
            description: "Chinese male - confident broadcaster tone".to_string(),
            category: VoiceCategory::Male,
            preview_audio: None,
        },
        Voice {
            id: "zm_yunyang".to_string(),
            name: "云扬 (Yunyang)".to_string(),
            description: "Chinese male - young and energetic".to_string(),
            category: VoiceCategory::Male,
            preview_audio: None,
        },
        Voice {
            id: "zf_xiaoni".to_string(),
            name: "小妮 (Xiaoni)".to_string(),
            description: "Chinese female - warm narrative voice".to_string(),
            category: VoiceCategory::Female,
            preview_audio: None,
        },
        Voice {
            id: "zf_xiaoyi".to_string(),
            name: "小艺 (Xiaoyi)".to_string(),
            description: "Chinese female - professional and authoritative".to_string(),
            category: VoiceCategory::Female,
            preview_audio: None,
        },
        // English voices
        Voice {
            id: "af_heart".to_string(),
            name: "Heart".to_string(),
            description: "American female - warm and friendly".to_string(),
            category: VoiceCategory::Female,
            preview_audio: None,
        },
        Voice {
            id: "af_bella".to_string(),
            name: "Bella".to_string(),
            description: "American female - popular and expressive".to_string(),
            category: VoiceCategory::Female,
            preview_audio: None,
        },
        Voice {
            id: "am_adam".to_string(),
            name: "Adam".to_string(),
            description: "American male - clear and professional".to_string(),
            category: VoiceCategory::Male,
            preview_audio: None,
        },
        Voice {
            id: "am_michael".to_string(),
            name: "Michael".to_string(),
            description: "American male - natural and versatile".to_string(),
            category: VoiceCategory::Male,
            preview_audio: None,
        },
        Voice {
            id: "bf_emma".to_string(),
            name: "Emma".to_string(),
            description: "British female - elegant and refined".to_string(),
            category: VoiceCategory::Female,
            preview_audio: None,
        },
        Voice {
            id: "bm_george".to_string(),
            name: "George".to_string(),
            description: "British male - distinguished and articulate".to_string(),
            category: VoiceCategory::Male,
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
