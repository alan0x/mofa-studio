//! Voice Clone Modal - UI for creating custom voices via zero-shot cloning
//!
//! Supports two modes:
//! 1. Select existing audio file + manually enter prompt text
//! 2. Record voice via microphone + auto-transcribe with ASR

use crate::audio_player::TTSPlayer;
use crate::voice_data::{CloningStatus, Voice};
use crate::voice_persistence;
use makepad_widgets::*;
use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Recording state
#[derive(Clone, Debug, PartialEq, Default)]
pub enum RecordingStatus {
    #[default]
    Idle,
    Recording,
    Transcribing,
    Completed,
    Error(String),
}

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use mofa_widgets::theme::*;

    // Modal overlay background
    ModalOverlay = <View> {
        width: Fill, height: Fill
        show_bg: true
        draw_bg: {
            fn pixel(self) -> vec4 {
                return vec4(0.0, 0.0, 0.0, 0.5);
            }
        }
    }

    // Text input field with label
    LabeledInput = <View> {
        width: Fill, height: Fit
        flow: Down
        spacing: 6

        label = <Label> {
            width: Fill, height: Fit
            draw_text: {
                instance dark_mode: 0.0
                text_style: <FONT_SEMIBOLD>{ font_size: 12.0 }
                fn get_color(self) -> vec4 {
                    return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                }
            }
        }

        input = <TextInput> {
            width: Fill, height: 40
            padding: {left: 12, right: 12, top: 8, bottom: 8}
            empty_text: ""
            ascii_only: false

            draw_bg: {
                instance dark_mode: 0.0
                border_radius: 6.0
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                    let bg = mix((WHITE), (SLATE_700), self.dark_mode);
                    let border = mix((SLATE_200), (SLATE_600), self.dark_mode);
                    sdf.fill(bg);
                    sdf.stroke(border, 1.0);
                    return sdf.result;
                }
            }

            draw_text: {
                instance dark_mode: 0.0
                text_style: { font_size: 13.0 }
                fn get_color(self) -> vec4 {
                    return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                }
            }

            draw_cursor: {
                instance focus: 0.0
                uniform border_radius: 0.5
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, self.border_radius);
                    sdf.fill(mix((PRIMARY_500), (PRIMARY_500), self.focus));
                    return sdf.result;
                }
            }

            draw_selection: {
                instance focus: 0.0
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 1.0);
                    sdf.fill(mix(vec4(0.23, 0.51, 0.97, 0.2), vec4(0.23, 0.51, 0.97, 0.35), self.focus));
                    return sdf.result;
                }
            }
        }
    }

    // File selector row with record option
    FileSelector = <View> {
        width: Fill, height: Fit
        flow: Down
        spacing: 6

        label = <Label> {
            width: Fill, height: Fit
            draw_text: {
                instance dark_mode: 0.0
                text_style: <FONT_SEMIBOLD>{ font_size: 12.0 }
                fn get_color(self) -> vec4 {
                    return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                }
            }
            text: "Reference Audio (3-10 seconds)"
        }

        file_row = <View> {
            width: Fill, height: 40
            flow: Right
            spacing: 8
            align: {y: 0.5}

            // Record button (microphone)
            record_btn = <Button> {
                width: 36, height: 36

                draw_bg: {
                    instance dark_mode: 0.0
                    instance hover: 0.0
                    instance recording: 0.0

                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        sdf.circle(18.0, 18.0, 17.0);

                        // Background
                        let base = mix((SLATE_100), (SLATE_700), self.dark_mode);
                        let hover_color = mix((RED_100), (RED_900), self.dark_mode);
                        let recording_color = mix((RED_500), (RED_400), self.dark_mode);
                        let color = mix(base, hover_color, self.hover * (1.0 - self.recording));
                        let color = mix(color, recording_color, self.recording);
                        sdf.fill(color);

                        // Microphone icon or stop square
                        if self.recording > 0.5 {
                            // Stop icon (square)
                            sdf.rect(13.0, 13.0, 10.0, 10.0);
                            sdf.fill((WHITE));
                        } else {
                            // Microphone icon (simplified)
                            let icon_color = mix((SLATE_500), (SLATE_400), self.dark_mode);
                            let icon_color = mix(icon_color, (RED_500), self.hover);
                            // Mic body
                            sdf.box(15.0, 10.0, 6.0, 10.0, 3.0);
                            sdf.fill(icon_color);
                            // Mic stand arc
                            sdf.move_to(12.0, 18.0);
                            sdf.line_to(12.0, 20.0);
                            sdf.line_to(18.0, 24.0);
                            sdf.line_to(24.0, 20.0);
                            sdf.line_to(24.0, 18.0);
                            sdf.stroke(icon_color, 1.5);
                            // Mic stand
                            sdf.move_to(18.0, 24.0);
                            sdf.line_to(18.0, 27.0);
                            sdf.stroke(icon_color, 1.5);
                        }

                        return sdf.result;
                    }
                }

                draw_text: {
                    text_style: { font_size: 0.0 }
                    fn get_color(self) -> vec4 {
                        return vec4(0.0, 0.0, 0.0, 0.0);
                    }
                }
            }

            select_btn = <Button> {
                width: Fit, height: 36
                padding: {left: 16, right: 16}
                text: "Select File..."

                draw_bg: {
                    instance dark_mode: 0.0
                    instance hover: 0.0
                    border_radius: 6.0
                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                        let base = mix((SLATE_100), (SLATE_700), self.dark_mode);
                        let hover_color = mix((SLATE_200), (SLATE_600), self.dark_mode);
                        sdf.fill(mix(base, hover_color, self.hover));
                        sdf.stroke(mix((SLATE_300), (SLATE_500), self.dark_mode), 1.0);
                        return sdf.result;
                    }
                }

                draw_text: {
                    instance dark_mode: 0.0
                    text_style: { font_size: 12.0 }
                    fn get_color(self) -> vec4 {
                        return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                    }
                }
            }

            file_name = <Label> {
                width: Fill, height: Fit
                draw_text: {
                    instance dark_mode: 0.0
                    text_style: { font_size: 12.0 }
                    fn get_color(self) -> vec4 {
                        return mix((TEXT_TERTIARY), (TEXT_TERTIARY_DARK), self.dark_mode);
                    }
                }
                text: "No file selected"
            }

            // Preview button
            preview_btn = <Button> {
                width: 36, height: 36
                visible: false

                draw_bg: {
                    instance dark_mode: 0.0
                    instance hover: 0.0
                    instance playing: 0.0

                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        sdf.circle(18.0, 18.0, 17.0);
                        let base = mix((SLATE_100), (SLATE_700), self.dark_mode);
                        let hover_color = mix((PRIMARY_100), (PRIMARY_700), self.dark_mode);
                        let color = mix(base, hover_color, self.hover);
                        sdf.fill(color);

                        // Draw play triangle
                        if self.playing > 0.5 {
                            // Stop icon (square)
                            sdf.rect(13.0, 13.0, 10.0, 10.0);
                            let icon_color = mix((PRIMARY_600), (PRIMARY_300), self.dark_mode);
                            sdf.fill(icon_color);
                        } else {
                            // Play icon (triangle)
                            sdf.move_to(14.0, 11.0);
                            sdf.line_to(25.0, 18.0);
                            sdf.line_to(14.0, 25.0);
                            sdf.close_path();
                            let icon_color = mix((SLATE_500), (SLATE_400), self.dark_mode);
                            sdf.fill(mix(icon_color, (PRIMARY_500), self.hover));
                        }

                        return sdf.result;
                    }
                }

                draw_text: {
                    text_style: { font_size: 0.0 }
                    fn get_color(self) -> vec4 {
                        return vec4(0.0, 0.0, 0.0, 0.0);
                    }
                }
            }
        }

        audio_info = <Label> {
            width: Fill, height: Fit
            margin: { top: 4 }
            draw_text: {
                instance dark_mode: 0.0
                text_style: { font_size: 11.0 }
                fn get_color(self) -> vec4 {
                    return mix((TEXT_TERTIARY), (TEXT_TERTIARY_DARK), self.dark_mode);
                }
            }
            text: ""
        }
    }

    // Language dropdown
    LanguageSelector = <View> {
        width: Fill, height: Fit
        flow: Down
        spacing: 6

        label = <Label> {
            width: Fill, height: Fit
            draw_text: {
                instance dark_mode: 0.0
                text_style: <FONT_SEMIBOLD>{ font_size: 12.0 }
                fn get_color(self) -> vec4 {
                    return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                }
            }
            text: "Language"
        }

        lang_row = <View> {
            width: Fill, height: Fit
            flow: Right
            spacing: 12

            zh_btn = <Button> {
                width: Fit, height: 36
                padding: {left: 20, right: 20}
                text: "Chinese"

                draw_bg: {
                    instance dark_mode: 0.0
                    instance hover: 0.0
                    instance selected: 1.0
                    border_radius: 6.0
                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                        let base = mix((SLATE_100), (SLATE_700), self.dark_mode);
                        let selected_color = mix((PRIMARY_100), (PRIMARY_800), self.dark_mode);
                        let hover_color = mix((SLATE_200), (SLATE_600), self.dark_mode);
                        let color = mix(base, selected_color, self.selected);
                        let color = mix(color, hover_color, self.hover * (1.0 - self.selected));
                        sdf.fill(color);
                        let border = mix(mix((SLATE_300), (SLATE_500), self.dark_mode), (PRIMARY_500), self.selected);
                        sdf.stroke(border, 1.0);
                        return sdf.result;
                    }
                }

                draw_text: {
                    instance dark_mode: 0.0
                    instance selected: 1.0
                    text_style: { font_size: 12.0 }
                    fn get_color(self) -> vec4 {
                        let base = mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                        let selected = mix((PRIMARY_700), (PRIMARY_200), self.dark_mode);
                        return mix(base, selected, self.selected);
                    }
                }
            }

            en_btn = <Button> {
                width: Fit, height: 36
                padding: {left: 20, right: 20}
                text: "English"

                draw_bg: {
                    instance dark_mode: 0.0
                    instance hover: 0.0
                    instance selected: 0.0
                    border_radius: 6.0
                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                        let base = mix((SLATE_100), (SLATE_700), self.dark_mode);
                        let selected_color = mix((PRIMARY_100), (PRIMARY_800), self.dark_mode);
                        let hover_color = mix((SLATE_200), (SLATE_600), self.dark_mode);
                        let color = mix(base, selected_color, self.selected);
                        let color = mix(color, hover_color, self.hover * (1.0 - self.selected));
                        sdf.fill(color);
                        let border = mix(mix((SLATE_300), (SLATE_500), self.dark_mode), (PRIMARY_500), self.selected);
                        sdf.stroke(border, 1.0);
                        return sdf.result;
                    }
                }

                draw_text: {
                    instance dark_mode: 0.0
                    instance selected: 0.0
                    text_style: { font_size: 12.0 }
                    fn get_color(self) -> vec4 {
                        let base = mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                        let selected = mix((PRIMARY_700), (PRIMARY_200), self.dark_mode);
                        return mix(base, selected, self.selected);
                    }
                }
            }
        }
    }

    // Progress log area (compact)
    ProgressLog = <View> {
        width: Fill, height: 100
        flow: Down
        spacing: 4

        label = <Label> {
            width: Fill, height: Fit
            draw_text: {
                instance dark_mode: 0.0
                text_style: <FONT_SEMIBOLD>{ font_size: 12.0 }
                fn get_color(self) -> vec4 {
                    return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                }
            }
            text: "Progress"
        }

        log_scroll = <ScrollYView> {
            width: Fill, height: Fill
            show_bg: true
            draw_bg: {
                instance dark_mode: 0.0
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                    let bg = mix((SLATE_50), (SLATE_800), self.dark_mode);
                    sdf.fill(bg);
                    sdf.stroke(mix((SLATE_200), (SLATE_600), self.dark_mode), 1.0);
                    return sdf.result;
                }
            }

            log_content = <Label> {
                width: Fill, height: Fit
                padding: {left: 10, right: 10, top: 8, bottom: 8}
                draw_text: {
                    instance dark_mode: 0.0
                    text_style: { font_size: 11.0, line_spacing: 1.5 }
                    fn get_color(self) -> vec4 {
                        return mix((SLATE_600), (SLATE_300), self.dark_mode);
                    }
                }
                text: "Ready to clone voice..."
            }
        }
    }

    // Action button
    ActionButton = <Button> {
        width: Fit, height: 40
        padding: {left: 24, right: 24}

        draw_bg: {
            instance dark_mode: 0.0
            instance hover: 0.0
            instance pressed: 0.0
            instance primary: 0.0
            border_radius: 6.0

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);

                // Primary button style
                let primary_base = mix((PRIMARY_500), (PRIMARY_400), self.dark_mode);
                let primary_hover = mix((PRIMARY_600), (PRIMARY_300), self.dark_mode);
                let primary_pressed = mix((PRIMARY_700), (PRIMARY_500), self.dark_mode);

                // Secondary button style
                let secondary_base = mix((SLATE_100), (SLATE_700), self.dark_mode);
                let secondary_hover = mix((SLATE_200), (SLATE_600), self.dark_mode);
                let secondary_pressed = mix((SLATE_300), (SLATE_500), self.dark_mode);

                let base = mix(secondary_base, primary_base, self.primary);
                let hover_color = mix(secondary_hover, primary_hover, self.primary);
                let pressed_color = mix(secondary_pressed, primary_pressed, self.primary);

                let color = mix(base, hover_color, self.hover);
                let color = mix(color, pressed_color, self.pressed);

                sdf.fill(color);

                // Border for secondary
                if self.primary < 0.5 {
                    sdf.stroke(mix((SLATE_300), (SLATE_500), self.dark_mode), 1.0);
                }

                return sdf.result;
            }
        }

        draw_text: {
            instance dark_mode: 0.0
            instance primary: 0.0
            text_style: <FONT_SEMIBOLD>{ font_size: 13.0 }
            fn get_color(self) -> vec4 {
                let secondary = mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                return mix(secondary, (WHITE), self.primary);
            }
        }
    }

    // Main modal dialog
    pub VoiceCloneModal = {{VoiceCloneModal}} {
        width: Fill, height: Fill
        flow: Overlay
        visible: false

        // Overlay background
        overlay = <ModalOverlay> {}

        // Modal container (scrollable when window is small)
        modal_container = <ScrollYView> {
            width: Fill, height: Fill
            align: {x: 0.5, y: 0.0}
            padding: {top: 40, bottom: 40}
            scroll_bars: <ScrollBars> {
                show_scroll_x: false
                show_scroll_y: true
            }

            // Centering wrapper
            modal_wrapper = <View> {
                width: Fill, height: Fit
                align: {x: 0.5, y: 0.0}

            // Modal content
            modal_content = <RoundedView> {
                width: 520, height: Fit
                flow: Down
                padding: 0

                draw_bg: {
                    instance dark_mode: 0.0
                    border_radius: 12.0
                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                        let bg = mix((WHITE), (SLATE_800), self.dark_mode);
                        sdf.fill(bg);
                        return sdf.result;
                    }
                }

                // Header
                header = <View> {
                    width: Fill, height: Fit
                    padding: {left: 24, right: 24, top: 20, bottom: 16}
                    flow: Right
                    align: {y: 0.5}

                    show_bg: true
                    draw_bg: {
                        instance dark_mode: 0.0
                        fn pixel(self) -> vec4 {
                            return mix((SLATE_50), (SLATE_700), self.dark_mode);
                        }
                    }

                    title = <Label> {
                        width: Fill, height: Fit
                        draw_text: {
                            instance dark_mode: 0.0
                            text_style: <FONT_BOLD>{ font_size: 16.0 }
                            fn get_color(self) -> vec4 {
                                return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                            }
                        }
                        text: "Clone Voice"
                    }

                    close_btn = <Button> {
                        width: 32, height: 32

                        draw_bg: {
                            instance dark_mode: 0.0
                            instance hover: 0.0
                            fn pixel(self) -> vec4 {
                                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                sdf.circle(16.0, 16.0, 15.0);
                                let base = mix((SLATE_100), (SLATE_600), self.dark_mode);
                                let hover_color = mix((SLATE_200), (SLATE_500), self.dark_mode);
                                sdf.fill(mix(base, hover_color, self.hover));

                                // X icon
                                let x_color = mix((SLATE_500), (SLATE_300), self.dark_mode);
                                sdf.move_to(11.0, 11.0);
                                sdf.line_to(21.0, 21.0);
                                sdf.stroke(x_color, 1.5);
                                sdf.move_to(21.0, 11.0);
                                sdf.line_to(11.0, 21.0);
                                sdf.stroke(x_color, 1.5);

                                return sdf.result;
                            }
                        }

                        draw_text: {
                            text_style: { font_size: 0.0 }
                            fn get_color(self) -> vec4 {
                                return vec4(0.0, 0.0, 0.0, 0.0);
                            }
                        }
                    }
                }

                // Body
                body = <View> {
                    width: Fill, height: Fit
                    flow: Down
                    padding: {left: 24, right: 24, top: 16, bottom: 16}
                    spacing: 16

                    // File selector
                    file_selector = <FileSelector> {}

                    // Reference text input
                    prompt_text_input = <LabeledInput> {
                        label = { text: "Reference Text (what the audio says)" }
                        input = {
                            height: 60
                            empty_text: "Enter the exact text spoken in the reference audio..."
                        }
                    }

                    // Voice name input
                    voice_name_input = <LabeledInput> {
                        label = { text: "Voice Name" }
                        input = {
                            empty_text: "Enter a name for this voice..."
                        }
                    }

                    // Language selector
                    language_selector = <LanguageSelector> {}

                    // Progress log
                    progress_log = <ProgressLog> {}
                }

                // Footer with action buttons
                footer = <View> {
                    width: Fill, height: Fit
                    padding: {left: 24, right: 24, top: 16, bottom: 20}
                    flow: Right
                    align: {x: 1.0, y: 0.5}
                    spacing: 12

                    show_bg: true
                    draw_bg: {
                        instance dark_mode: 0.0
                        fn pixel(self) -> vec4 {
                            return mix((SLATE_50), (SLATE_700), self.dark_mode);
                        }
                    }

                    cancel_btn = <ActionButton> {
                        text: "Cancel"
                        draw_bg: { primary: 0.0 }
                        draw_text: { primary: 0.0 }
                    }

                    save_btn = <ActionButton> {
                        text: "Save Voice"
                        draw_bg: { primary: 1.0 }
                        draw_text: { primary: 1.0 }
                    }
                }
            } // end modal_content
            } // end modal_wrapper
        } // end modal_container
    }
}

/// Actions emitted by VoiceCloneModal
#[derive(Clone, Debug, DefaultNone)]
pub enum VoiceCloneModalAction {
    None,
    Closed,
    VoiceCreated(Voice),
}

#[derive(Live, LiveHook, Widget)]
pub struct VoiceCloneModal {
    #[deref]
    view: View,

    #[rust]
    dark_mode: f64,

    #[rust]
    selected_file: Option<PathBuf>,

    #[rust]
    audio_info: Option<voice_persistence::AudioInfo>,

    #[rust]
    selected_language: String,

    #[rust]
    cloning_status: CloningStatus,

    #[rust]
    log_messages: Vec<String>,

    #[rust]
    preview_player: Option<TTSPlayer>,

    #[rust]
    preview_playing: bool,

    // Recording state
    #[rust]
    recording_status: RecordingStatus,

    #[rust]
    recording_buffer: Arc<Mutex<Vec<f32>>>,

    #[rust]
    is_recording: Arc<AtomicBool>,

    #[rust]
    recording_start_time: Option<std::time::Instant>,

    #[rust]
    recorded_audio_path: Option<PathBuf>,

    #[rust]
    recording_sample_rate: Arc<Mutex<u32>>,

    #[rust]
    processing_complete: Arc<AtomicBool>,

    #[rust]
    temp_audio_file: Arc<Mutex<Option<PathBuf>>>,
}

impl Widget for VoiceCloneModal {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        // Initialize defaults
        if self.selected_language.is_empty() {
            self.selected_language = "zh".to_string();
        }

        // Check if recording processing is complete
        if self.processing_complete.load(Ordering::Relaxed) {
            self.processing_complete.store(false, Ordering::Relaxed);

            // Load the recorded audio file
            let path = {
                self.temp_audio_file.lock().take()
            };

            if let Some(path) = path {
                self.add_log(cx, "[INFO] Loading recorded audio...");
                self.handle_file_selected(cx, path);
                self.recording_status = RecordingStatus::Completed;
                self.add_log(cx, "[INFO] Recording complete! Please enter the reference text.");
            }
        }

        // Keep redrawing while processing to check for completion
        if self.recording_status == RecordingStatus::Transcribing {
            self.view.redraw(cx);
        }

        let actions = match event {
            Event::Actions(actions) => actions.as_slice(),
            _ => return,
        };

        // Handle close button
        if self
            .view
            .button(ids!(
                modal_container.modal_wrapper.modal_content.header.close_btn
            ))
            .clicked(actions)
        {
            self.close(cx, scope);
        }

        // Handle cancel button
        if self
            .view
            .button(ids!(
                modal_container
                    .modal_wrapper
                    .modal_content
                    .footer
                    .cancel_btn
            ))
            .clicked(actions)
        {
            self.close(cx, scope);
        }

        // Handle record button
        if self
            .view
            .button(ids!(
                modal_container
                    .modal_wrapper
                    .modal_content
                    .body
                    .file_selector
                    .file_row
                    .record_btn
            ))
            .clicked(actions)
        {
            self.toggle_recording(cx);
        }

        // Handle file select button
        if self
            .view
            .button(ids!(
                modal_container
                    .modal_wrapper
                    .modal_content
                    .body
                    .file_selector
                    .file_row
                    .select_btn
            ))
            .clicked(actions)
        {
            self.open_file_dialog(cx);
        }

        // Handle preview button
        if self
            .view
            .button(ids!(
                modal_container
                    .modal_wrapper
                    .modal_content
                    .body
                    .file_selector
                    .file_row
                    .preview_btn
            ))
            .clicked(actions)
        {
            self.toggle_preview(cx);
        }

        // Handle language buttons
        if self
            .view
            .button(ids!(
                modal_container
                    .modal_wrapper
                    .modal_content
                    .body
                    .language_selector
                    .lang_row
                    .zh_btn
            ))
            .clicked(actions)
        {
            self.selected_language = "zh".to_string();
            self.update_language_buttons(cx);
        }
        if self
            .view
            .button(ids!(
                modal_container
                    .modal_wrapper
                    .modal_content
                    .body
                    .language_selector
                    .lang_row
                    .en_btn
            ))
            .clicked(actions)
        {
            self.selected_language = "en".to_string();
            self.update_language_buttons(cx);
        }

        // Handle save button
        if self
            .view
            .button(ids!(
                modal_container.modal_wrapper.modal_content.footer.save_btn
            ))
            .clicked(actions)
        {
            self.save_voice(cx, scope);
        }

        // Handle overlay click to close
        let overlay = self.view.view(ids!(overlay));
        if let Hit::FingerUp(fe) = event.hits(cx, overlay.area()) {
            if !fe.is_over {
                // Click was on overlay, not content
            } else {
                // Check if click is outside modal content
                let modal_content = self
                    .view
                    .view(ids!(modal_container.modal_wrapper.modal_content));
                if !modal_content.area().rect(cx).contains(fe.abs) {
                    self.close(cx, scope);
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl VoiceCloneModal {
    fn add_log(&mut self, cx: &mut Cx, message: &str) {
        self.log_messages.push(message.to_string());
        let log_text = self.log_messages.join("\n");
        self.view
            .label(ids!(
                modal_container
                    .modal_wrapper
                    .modal_content
                    .body
                    .progress_log
                    .log_scroll
                    .log_content
            ))
            .set_text(cx, &log_text);
        self.view.redraw(cx);
    }

    fn clear_log(&mut self, cx: &mut Cx) {
        self.log_messages.clear();
        self.view
            .label(ids!(
                modal_container
                    .modal_wrapper
                    .modal_content
                    .body
                    .progress_log
                    .log_scroll
                    .log_content
            ))
            .set_text(cx, "Ready to clone voice...");
        self.view.redraw(cx);
    }

    fn open_file_dialog(&mut self, cx: &mut Cx) {
        // Use rfd for native file dialog
        let dialog = rfd::FileDialog::new()
            .add_filter("Audio Files", &["wav", "mp3", "flac", "ogg"])
            .add_filter("WAV Files", &["wav"])
            .set_title("Select Reference Audio");

        if let Some(path) = dialog.pick_file() {
            self.handle_file_selected(cx, path);
        }
    }

    fn handle_file_selected(&mut self, cx: &mut Cx, path: PathBuf) {
        // Update file name label
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown");
        self.view
            .label(ids!(
                modal_container
                    .modal_wrapper
                    .modal_content
                    .body
                    .file_selector
                    .file_row
                    .file_name
            ))
            .set_text(cx, file_name);

        // Validate audio file
        self.add_log(cx, "[INFO] Validating audio file...");

        match voice_persistence::validate_audio_file(&path) {
            Ok(info) => {
                self.add_log(
                    cx,
                    &format!(
                        "[INFO] Audio OK: {:.1}s, {}Hz, {} channels",
                        info.duration_secs, info.sample_rate, info.channels
                    ),
                );

                for warning in &info.warnings {
                    self.add_log(cx, &format!("[WARN] {}", warning));
                }

                // Update audio info label
                let info_text = format!(
                    "Duration: {:.1}s | Sample rate: {}Hz | Channels: {}",
                    info.duration_secs, info.sample_rate, info.channels
                );
                self.view
                    .label(ids!(
                        modal_container
                            .modal_wrapper
                            .modal_content
                            .body
                            .file_selector
                            .audio_info
                    ))
                    .set_text(cx, &info_text);

                self.audio_info = Some(info);
                self.selected_file = Some(path);

                // Show preview button
                self.view
                    .button(ids!(
                        modal_container
                            .modal_wrapper
                            .modal_content
                            .body
                            .file_selector
                            .file_row
                            .preview_btn
                    ))
                    .set_visible(cx, true);
            }
            Err(e) => {
                self.add_log(cx, &format!("[ERROR] {}", e));
                self.selected_file = None;
                self.audio_info = None;
                self.view
                    .button(ids!(
                        modal_container
                            .modal_wrapper
                            .modal_content
                            .body
                            .file_selector
                            .file_row
                            .preview_btn
                    ))
                    .set_visible(cx, false);
            }
        }

        self.view.redraw(cx);
    }

    fn toggle_preview(&mut self, cx: &mut Cx) {
        if self.preview_playing {
            // Stop preview
            if let Some(player) = &self.preview_player {
                player.stop();
            }
            self.preview_playing = false;
            self.update_preview_button(cx, false);
            return;
        }

        // Play preview
        if let Some(path) = &self.selected_file {
            // Initialize player if needed
            if self.preview_player.is_none() {
                self.preview_player = Some(TTSPlayer::new());
            }

            // Load and play audio
            match self.load_wav_for_preview(path) {
                Ok(samples) => {
                    if let Some(player) = &self.preview_player {
                        player.write_audio(&samples);
                    }
                    self.preview_playing = true;
                    self.update_preview_button(cx, true);
                    self.add_log(cx, "[INFO] Playing preview...");
                }
                Err(e) => {
                    self.add_log(cx, &format!("[ERROR] Failed to play: {}", e));
                }
            }
        }
    }

    fn load_wav_for_preview(&self, path: &PathBuf) -> Result<Vec<f32>, String> {
        use hound::WavReader;

        let reader = WavReader::open(path).map_err(|e| format!("Failed to open WAV: {}", e))?;

        let spec = reader.spec();
        let sample_rate = spec.sample_rate;
        let channels = spec.channels as usize;

        // Read samples
        let samples: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Int => {
                let bits = spec.bits_per_sample;
                let max_val = (1 << (bits - 1)) as f32;
                reader
                    .into_samples::<i32>()
                    .filter_map(Result::ok)
                    .map(|s| s as f32 / max_val)
                    .collect()
            }
            hound::SampleFormat::Float => reader
                .into_samples::<f32>()
                .filter_map(Result::ok)
                .collect(),
        };

        // Convert to mono
        let mono_samples: Vec<f32> = if channels > 1 {
            samples
                .chunks(channels)
                .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
                .collect()
        } else {
            samples
        };

        // Resample to 32000 Hz if needed
        let target_rate = 32000;
        let resampled = if sample_rate != target_rate {
            let ratio = target_rate as f32 / sample_rate as f32;
            let new_len = (mono_samples.len() as f32 * ratio) as usize;
            let mut result = Vec::with_capacity(new_len);
            for i in 0..new_len {
                let src_idx = i as f32 / ratio;
                let idx = src_idx as usize;
                let frac = src_idx - idx as f32;
                let s1 = mono_samples.get(idx).copied().unwrap_or(0.0);
                let s2 = mono_samples.get(idx + 1).copied().unwrap_or(s1);
                result.push(s1 + (s2 - s1) * frac);
            }
            result
        } else {
            mono_samples
        };

        Ok(resampled)
    }

    fn update_preview_button(&mut self, cx: &mut Cx, playing: bool) {
        let playing_val = if playing { 1.0 } else { 0.0 };
        self.view
            .button(ids!(
                modal_container
                    .modal_wrapper
                    .modal_content
                    .body
                    .file_selector
                    .file_row
                    .preview_btn
            ))
            .apply_over(
                cx,
                live! {
                    draw_bg: { playing: (playing_val) }
                },
            );
        self.view.redraw(cx);
    }

    fn update_language_buttons(&mut self, cx: &mut Cx) {
        let zh_selected = if self.selected_language == "zh" {
            1.0
        } else {
            0.0
        };
        let en_selected = if self.selected_language == "en" {
            1.0
        } else {
            0.0
        };

        self.view
            .button(ids!(
                modal_container
                    .modal_wrapper
                    .modal_content
                    .body
                    .language_selector
                    .lang_row
                    .zh_btn
            ))
            .apply_over(
                cx,
                live! {
                    draw_bg: { selected: (zh_selected) }
                    draw_text: { selected: (zh_selected) }
                },
            );

        self.view
            .button(ids!(
                modal_container
                    .modal_wrapper
                    .modal_content
                    .body
                    .language_selector
                    .lang_row
                    .en_btn
            ))
            .apply_over(
                cx,
                live! {
                    draw_bg: { selected: (en_selected) }
                    draw_text: { selected: (en_selected) }
                },
            );

        self.view.redraw(cx);
    }

    fn save_voice(&mut self, cx: &mut Cx, scope: &mut Scope) {
        // Validate inputs
        let voice_name = self
            .view
            .text_input(ids!(
                modal_container
                    .modal_wrapper
                    .modal_content
                    .body
                    .voice_name_input
                    .input
            ))
            .text();

        if voice_name.trim().is_empty() {
            self.add_log(cx, "[ERROR] Please enter a voice name");
            return;
        }

        let prompt_text = self
            .view
            .text_input(ids!(
                modal_container
                    .modal_wrapper
                    .modal_content
                    .body
                    .prompt_text_input
                    .input
            ))
            .text();

        if prompt_text.trim().is_empty() {
            self.add_log(cx, "[ERROR] Please enter the reference text");
            return;
        }

        let source_path = match &self.selected_file {
            Some(p) => p.clone(),
            None => {
                self.add_log(cx, "[ERROR] Please select a reference audio file");
                return;
            }
        };

        // Validate audio duration (GPT-SoVITS requires 3-10 seconds)
        if let Some(ref info) = self.audio_info {
            if info.duration_secs < 3.0 {
                self.add_log(
                    cx,
                    &format!(
                        "[ERROR] Audio too short ({:.1}s). GPT-SoVITS requires 3-10 seconds.",
                        info.duration_secs
                    ),
                );
                self.add_log(cx, "[ERROR] Please select a longer audio file.");
                return;
            }
            if info.duration_secs > 10.0 {
                self.add_log(
                    cx,
                    &format!(
                        "[ERROR] Audio too long ({:.1}s). GPT-SoVITS requires 3-10 seconds.",
                        info.duration_secs
                    ),
                );
                self.add_log(cx, "[ERROR] Please select a shorter audio file or trim it.");
                return;
            }
        } else {
            self.add_log(
                cx,
                "[ERROR] Audio file not validated. Please re-select the file.",
            );
            return;
        }

        self.cloning_status = CloningStatus::ValidatingAudio;
        self.add_log(cx, "[INFO] Starting voice creation...");

        // Generate unique voice ID
        let voice_id = voice_persistence::generate_voice_id(&voice_name);
        self.add_log(cx, &format!("[INFO] Voice ID: {}", voice_id));

        // Copy audio file
        self.cloning_status = CloningStatus::CopyingFiles;
        self.add_log(cx, "[INFO] Copying reference audio...");

        let relative_path = match voice_persistence::copy_reference_audio(&voice_id, &source_path) {
            Ok(path) => path,
            Err(e) => {
                self.add_log(cx, &format!("[ERROR] {}", e));
                self.cloning_status = CloningStatus::Error(e);
                return;
            }
        };

        self.add_log(cx, "[INFO] Audio file copied successfully");

        // Create voice object
        let voice = Voice::new_custom(
            voice_id.clone(),
            voice_name.trim().to_string(),
            self.selected_language.clone(),
            relative_path,
            prompt_text.trim().to_string(),
        );

        // Save to config
        self.cloning_status = CloningStatus::SavingConfig;
        self.add_log(cx, "[INFO] Saving voice configuration...");

        match voice_persistence::add_custom_voice(voice.clone()) {
            Ok(_) => {
                self.add_log(cx, "");
                self.add_log(cx, "✓ Voice created successfully!");
                self.add_log(cx, "You can now close this dialog.");
                self.cloning_status = CloningStatus::Completed;

                // Update save button to show completion
                self.view
                    .button(ids!(
                        modal_container.modal_wrapper.modal_content.footer.save_btn
                    ))
                    .set_text(cx, "✓ Done");

                // Emit action
                cx.widget_action(
                    self.widget_uid(),
                    &scope.path,
                    VoiceCloneModalAction::VoiceCreated(voice),
                );

                // Note: Don't auto-close, let user see the completion status
                // User can close via Cancel button or X button
            }
            Err(e) => {
                self.add_log(cx, &format!("[ERROR] Failed to save: {}", e));
                self.cloning_status = CloningStatus::Error(e);
            }
        }
    }

    fn close(&mut self, cx: &mut Cx, scope: &mut Scope) {
        // Stop any recording
        if self.is_recording.load(Ordering::Relaxed) {
            self.is_recording.store(false, Ordering::Relaxed);
        }
        self.recording_status = RecordingStatus::Idle;

        // Stop any preview playing
        if let Some(player) = &self.preview_player {
            player.stop();
        }
        self.preview_playing = false;

        // Reset state
        self.selected_file = None;
        self.audio_info = None;
        self.cloning_status = CloningStatus::Idle;
        self.recorded_audio_path = None;
        self.clear_log(cx);

        // Reset form fields
        self.view
            .text_input(ids!(
                modal_container
                    .modal_wrapper
                    .modal_content
                    .body
                    .voice_name_input
                    .input
            ))
            .set_text(cx, "");
        self.view
            .text_input(ids!(
                modal_container
                    .modal_wrapper
                    .modal_content
                    .body
                    .prompt_text_input
                    .input
            ))
            .set_text(cx, "");
        self.view
            .label(ids!(
                modal_container
                    .modal_wrapper
                    .modal_content
                    .body
                    .file_selector
                    .file_row
                    .file_name
            ))
            .set_text(cx, "No file selected");
        self.view
            .label(ids!(
                modal_container
                    .modal_wrapper
                    .modal_content
                    .body
                    .file_selector
                    .audio_info
            ))
            .set_text(cx, "");
        self.view
            .button(ids!(
                modal_container
                    .modal_wrapper
                    .modal_content
                    .body
                    .file_selector
                    .file_row
                    .preview_btn
            ))
            .set_visible(cx, false);

        // Reset record button
        self.update_record_button(cx, false);

        // Hide modal
        self.view.set_visible(cx, false);

        // Emit closed action
        cx.widget_action(
            self.widget_uid(),
            &scope.path,
            VoiceCloneModalAction::Closed,
        );
    }

    fn toggle_recording(&mut self, cx: &mut Cx) {
        if self.is_recording.load(Ordering::Relaxed) {
            self.stop_recording(cx);
        } else {
            self.start_recording(cx);
        }
    }

    fn start_recording(&mut self, cx: &mut Cx) {
        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

        self.add_log(cx, "[INFO] Starting microphone recording...");
        self.add_log(cx, "[INFO] Speak clearly for 3-10 seconds");

        // Initialize buffer and sample rate
        self.recording_buffer = Arc::new(Mutex::new(Vec::new()));
        self.is_recording = Arc::new(AtomicBool::new(true));
        self.recording_sample_rate = Arc::new(Mutex::new(16000)); // Default, will be updated
        self.recording_start_time = Some(std::time::Instant::now());
        self.recording_status = RecordingStatus::Recording;

        // Update UI
        self.update_record_button(cx, true);

        // Start recording in background thread
        let buffer = Arc::clone(&self.recording_buffer);
        let is_recording = Arc::clone(&self.is_recording);
        let sample_rate_store = Arc::clone(&self.recording_sample_rate);

        std::thread::spawn(move || {
            let host = cpal::default_host();

            let device = match host.default_input_device() {
                Some(d) => d,
                None => {
                    eprintln!("[VoiceClone] No input device found");
                    is_recording.store(false, Ordering::Relaxed);
                    return;
                }
            };

            eprintln!("[VoiceClone] Using input device: {:?}", device.name());

            // Get device's default/supported config instead of forcing 16kHz
            let supported_config = match device.default_input_config() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[VoiceClone] Failed to get default input config: {}", e);
                    is_recording.store(false, Ordering::Relaxed);
                    return;
                }
            };

            let sample_rate = supported_config.sample_rate().0;
            let channels = supported_config.channels() as usize;
            eprintln!(
                "[VoiceClone] Using config: {}Hz, {} channels",
                sample_rate, channels
            );

            // Store the actual sample rate for later resampling
            *sample_rate_store.lock() = sample_rate;

            let config: cpal::StreamConfig = supported_config.into();

            let buffer_clone = Arc::clone(&buffer);
            let is_recording_clone = Arc::clone(&is_recording);

            // We'll store raw samples and resample later
            let stream = match device.build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if is_recording_clone.load(Ordering::Relaxed) {
                        // Convert to mono if stereo
                        if channels > 1 {
                            let mono: Vec<f32> = data
                                .chunks(channels)
                                .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
                                .collect();
                            buffer_clone.lock().extend_from_slice(&mono);
                        } else {
                            buffer_clone.lock().extend_from_slice(data);
                        }
                    }
                },
                |err| eprintln!("[VoiceClone] Recording error: {}", err),
                None,
            ) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[VoiceClone] Failed to build input stream: {}", e);
                    is_recording.store(false, Ordering::Relaxed);
                    return;
                }
            };

            if let Err(e) = stream.play() {
                eprintln!("[VoiceClone] Failed to start stream: {}", e);
                is_recording.store(false, Ordering::Relaxed);
                return;
            }

            eprintln!("[VoiceClone] Recording started at {}Hz", sample_rate);

            // Keep stream alive while recording (max 12 seconds)
            let max_duration = std::time::Duration::from_secs(12);
            let start = std::time::Instant::now();

            while is_recording.load(Ordering::Relaxed) && start.elapsed() < max_duration {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            // Auto-stop after max duration
            is_recording.store(false, Ordering::Relaxed);
            eprintln!("[VoiceClone] Recording stopped ({}Hz mono)", sample_rate);
        });

        self.view.redraw(cx);
    }

    fn stop_recording(&mut self, cx: &mut Cx) {
        self.is_recording.store(false, Ordering::Relaxed);
        self.update_record_button(cx, false);

        // Calculate duration
        let duration = self
            .recording_start_time
            .map(|t| t.elapsed().as_secs_f32())
            .unwrap_or(0.0);

        self.add_log(cx, &format!("[INFO] Recording stopped ({:.1}s)", duration));

        // Validate duration
        if duration < 3.0 {
            self.add_log(
                cx,
                "[ERROR] Recording too short. Please record at least 3 seconds.",
            );
            self.recording_status = RecordingStatus::Error("Recording too short".to_string());
            self.view.redraw(cx);
            return;
        }

        if duration > 10.0 {
            self.add_log(cx, "[WARN] Recording over 10s will be trimmed to 10s");
        }

        self.recording_status = RecordingStatus::Transcribing;
        self.add_log(cx, "[INFO] Processing recorded audio...");
        self.view.redraw(cx);

        // Process in background thread to avoid blocking UI
        let buffer = Arc::clone(&self.recording_buffer);
        let sample_rate_store = Arc::clone(&self.recording_sample_rate);
        let processing_complete = Arc::clone(&self.processing_complete);
        let temp_file_store = Arc::clone(&self.temp_audio_file);

        std::thread::spawn(move || {
            // Give the recording thread a moment to finalize
            std::thread::sleep(std::time::Duration::from_millis(300));

            // Get samples and sample rate
            let samples: Vec<f32> = {
                let buf = buffer.lock();
                buf.clone()
            };

            let source_sample_rate = *sample_rate_store.lock();

            if samples.is_empty() {
                eprintln!("[VoiceClone] No audio recorded");
                return;
            }

            let duration = samples.len() as f32 / source_sample_rate as f32;
            eprintln!(
                "[VoiceClone] Recorded {} samples at {}Hz ({:.1}s)",
                samples.len(),
                source_sample_rate,
                duration
            );

            // Resample to 16kHz if needed
            let target_sample_rate: u32 = 16000;
            let resampled: Vec<f32> = if source_sample_rate != target_sample_rate {
                eprintln!(
                    "[VoiceClone] Resampling {}Hz -> {}Hz",
                    source_sample_rate, target_sample_rate
                );
                Self::resample(&samples, source_sample_rate, target_sample_rate)
            } else {
                samples
            };

            // Trim to max 10 seconds
            let max_samples = (10 * target_sample_rate) as usize;
            let trimmed_samples: Vec<f32> = if resampled.len() > max_samples {
                resampled[..max_samples].to_vec()
            } else {
                resampled
            };

            let final_duration = trimmed_samples.len() as f32 / target_sample_rate as f32;
            eprintln!(
                "[VoiceClone] Final audio: {} samples ({:.1}s)",
                trimmed_samples.len(),
                final_duration
            );

            // Save to temp file
            let temp_dir = std::env::temp_dir();
            let temp_file =
                temp_dir.join(format!("voice_clone_recording_{}.wav", std::process::id()));

            if let Err(e) = Self::save_wav_static(&temp_file, &trimmed_samples, target_sample_rate) {
                eprintln!("[VoiceClone] Failed to save WAV: {}", e);
                return;
            }

            eprintln!("[VoiceClone] Audio saved to: {:?}", temp_file);

            // Store the temp file path and signal completion
            *temp_file_store.lock() = Some(temp_file.clone());
            processing_complete.store(true, Ordering::Relaxed);

            eprintln!("[VoiceClone] Processing complete. Please enter text manually.");
        });
    }

    fn process_recorded_audio(&mut self, cx: &mut Cx) {
        self.recording_status = RecordingStatus::Transcribing;
        self.add_log(cx, "[INFO] Processing recorded audio...");

        // Get samples and sample rate from buffer
        let samples: Vec<f32> = {
            let buffer = self.recording_buffer.lock();
            buffer.clone()
        };

        let source_sample_rate = *self.recording_sample_rate.lock();

        if samples.is_empty() {
            self.add_log(cx, "[ERROR] No audio recorded");
            self.recording_status = RecordingStatus::Error("No audio".to_string());
            return;
        }

        let duration = samples.len() as f32 / source_sample_rate as f32;
        self.add_log(
            cx,
            &format!(
                "[INFO] Recorded {} samples at {}Hz ({:.1}s)",
                samples.len(),
                source_sample_rate,
                duration
            ),
        );

        // Resample to 16kHz if needed
        let target_sample_rate: u32 = 16000;
        let resampled: Vec<f32> = if source_sample_rate != target_sample_rate {
            self.add_log(
                cx,
                &format!(
                    "[INFO] Resampling {}Hz -> {}Hz",
                    source_sample_rate, target_sample_rate
                ),
            );
            Self::resample(&samples, source_sample_rate, target_sample_rate)
        } else {
            samples
        };

        // Trim to max 10 seconds (at 16kHz)
        let max_samples = (10 * target_sample_rate) as usize;
        let trimmed_samples: Vec<f32> = if resampled.len() > max_samples {
            resampled[..max_samples].to_vec()
        } else {
            resampled
        };

        let final_duration = trimmed_samples.len() as f32 / target_sample_rate as f32;
        self.add_log(
            cx,
            &format!(
                "[INFO] Final audio: {} samples ({:.1}s)",
                trimmed_samples.len(),
                final_duration
            ),
        );

        // Save to temp file
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!("voice_clone_recording_{}.wav", std::process::id()));

        match self.save_wav(&temp_file, &trimmed_samples, target_sample_rate) {
            Ok(_) => {
                self.add_log(cx, "[INFO] Audio saved, starting transcription...");
                self.recorded_audio_path = Some(temp_file.clone());

                // Run transcription
                self.transcribe_audio(cx, &temp_file);
            }
            Err(e) => {
                self.add_log(cx, &format!("[ERROR] Failed to save audio: {}", e));
                self.recording_status = RecordingStatus::Error(e);
            }
        }

        self.view.redraw(cx);
    }

    /// Simple linear interpolation resampling
    fn resample(samples: &[f32], source_rate: u32, target_rate: u32) -> Vec<f32> {
        if source_rate == target_rate {
            return samples.to_vec();
        }

        let ratio = target_rate as f64 / source_rate as f64;
        let new_len = (samples.len() as f64 * ratio) as usize;
        let mut result = Vec::with_capacity(new_len);

        for i in 0..new_len {
            let src_idx = i as f64 / ratio;
            let idx = src_idx as usize;
            let frac = (src_idx - idx as f64) as f32;

            let s1 = samples.get(idx).copied().unwrap_or(0.0);
            let s2 = samples.get(idx + 1).copied().unwrap_or(s1);
            result.push(s1 + (s2 - s1) * frac);
        }

        result
    }

    /// Static version of save_wav for use in background threads
    fn save_wav_static(path: &PathBuf, samples: &[f32], sample_rate: u32) -> Result<(), String> {
        use hound::{SampleFormat, WavSpec, WavWriter};

        let spec = WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut writer =
            WavWriter::create(path, spec).map_err(|e| format!("Failed to create WAV: {}", e))?;

        for &sample in samples {
            let sample_i16 = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
            writer
                .write_sample(sample_i16)
                .map_err(|e| format!("Failed to write sample: {}", e))?;
        }

        writer
            .finalize()
            .map_err(|e| format!("Failed to finalize WAV: {}", e))?;

        Ok(())
    }

    fn save_wav(&self, path: &PathBuf, samples: &[f32], sample_rate: u32) -> Result<(), String> {
        Self::save_wav_static(path, samples, sample_rate)
    }

    fn transcribe_audio(&mut self, cx: &mut Cx, audio_path: &PathBuf) {
        self.add_log(cx, "[INFO] Running ASR transcription...");

        // Find the transcribe script
        let script_path = std::env::current_dir()
            .map(|p| p.join("apps/mofa-primespeech/scripts/transcribe_audio.py"))
            .unwrap_or_default();

        // Determine language based on selection
        let lang_arg = match self.selected_language.as_str() {
            "zh" => "zh",
            "en" => "en",
            _ => "auto",
        };

        // Try multiple ways to run Python
        let audio_path_str = audio_path.to_string_lossy().to_string();
        let script_path_str = script_path.to_string_lossy().to_string();

        // Try pixi first, then python directly
        let output = std::process::Command::new("pixi")
            .args([
                "run",
                "python",
                &script_path_str,
                &audio_path_str,
                "-l",
                lang_arg,
            ])
            .output()
            .or_else(|_| {
                // Fallback to python directly
                std::process::Command::new("python")
                    .args([&script_path_str, &audio_path_str, "-l", lang_arg])
                    .output()
            })
            .or_else(|_| {
                // Fallback to python3
                std::process::Command::new("python3")
                    .args([&script_path_str, &audio_path_str, "-l", lang_arg])
                    .output()
            });

        match output {
            Ok(result) => {
                if result.status.success() {
                    let stdout = String::from_utf8_lossy(&result.stdout);
                    self.add_log(cx, &format!("[DEBUG] ASR output: {}", stdout.trim()));

                    // Parse JSON result
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                        if let Some(text) = json.get("text").and_then(|v| v.as_str()) {
                            let text = text.trim();
                            if !text.is_empty() {
                                self.add_log(cx, &format!("[INFO] Transcription: {}", text));

                                // Auto-fill the prompt text field
                                self.view
                                    .text_input(ids!(
                                        modal_container
                                            .modal_wrapper
                                            .modal_content
                                            .body
                                            .prompt_text_input
                                            .input
                                    ))
                                    .set_text(cx, text);

                                // Update file info
                                self.handle_file_selected(cx, audio_path.clone());

                                self.recording_status = RecordingStatus::Completed;
                                self.add_log(cx, "[INFO] Recording and transcription complete!");
                            } else {
                                self.add_log(cx, "[WARN] Transcription returned empty text");
                                self.recording_status =
                                    RecordingStatus::Error("Empty transcription".to_string());
                            }
                        } else if let Some(error) = json.get("error").and_then(|v| v.as_str()) {
                            self.add_log(cx, &format!("[ERROR] ASR error: {}", error));
                            self.recording_status = RecordingStatus::Error(error.to_string());
                        }
                    } else {
                        self.add_log(
                            cx,
                            &format!("[ERROR] Failed to parse ASR output: {}", stdout),
                        );
                        self.recording_status = RecordingStatus::Error("Parse error".to_string());
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    self.add_log(cx, &format!("[ERROR] ASR failed: {}", stderr));
                    self.recording_status = RecordingStatus::Error("ASR failed".to_string());

                    // Still use the recorded audio, user can manually enter text
                    self.handle_file_selected(cx, audio_path.clone());
                    self.add_log(cx, "[INFO] Audio saved. Please enter the text manually.");
                }
            }
            Err(e) => {
                self.add_log(cx, &format!("[ERROR] Failed to run ASR: {}", e));
                self.add_log(
                    cx,
                    "[INFO] Make sure 'pixi' is in PATH and ASR dependencies are installed",
                );
                self.recording_status = RecordingStatus::Error("ASR unavailable".to_string());

                // Still use the recorded audio
                self.handle_file_selected(cx, audio_path.clone());
                self.add_log(cx, "[INFO] Audio saved. Please enter the text manually.");
            }
        }

        self.view.redraw(cx);
    }

    fn update_record_button(&mut self, cx: &mut Cx, recording: bool) {
        let recording_val = if recording { 1.0 } else { 0.0 };
        self.view
            .button(ids!(
                modal_container
                    .modal_wrapper
                    .modal_content
                    .body
                    .file_selector
                    .file_row
                    .record_btn
            ))
            .apply_over(
                cx,
                live! {
                    draw_bg: { recording: (recording_val) }
                },
            );
        self.view.redraw(cx);
    }
}

impl VoiceCloneModalRef {
    /// Show the modal
    pub fn show(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.view.set_visible(cx, true);
            inner.clear_log(cx);
            inner.update_language_buttons(cx);
            inner.view.redraw(cx);
        }
    }

    /// Hide the modal
    pub fn hide(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.view.set_visible(cx, false);
        }
    }

    /// Update dark mode
    pub fn update_dark_mode(&self, cx: &mut Cx, dark_mode: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.dark_mode = dark_mode;

            // Apply to modal content
            inner
                .view
                .view(ids!(modal_container.modal_wrapper.modal_content))
                .apply_over(
                    cx,
                    live! {
                        draw_bg: { dark_mode: (dark_mode) }
                    },
                );

            // Apply to header
            inner
                .view
                .view(ids!(modal_container.modal_wrapper.modal_content.header))
                .apply_over(
                    cx,
                    live! {
                        draw_bg: { dark_mode: (dark_mode) }
                    },
                );

            // Apply to footer
            inner
                .view
                .view(ids!(modal_container.modal_wrapper.modal_content.footer))
                .apply_over(
                    cx,
                    live! {
                        draw_bg: { dark_mode: (dark_mode) }
                    },
                );

            inner.view.redraw(cx);
        }
    }
}
