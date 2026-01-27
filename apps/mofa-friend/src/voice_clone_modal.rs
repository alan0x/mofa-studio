//! Voice Clone Modal - UI for creating custom voices via zero-shot cloning

use crate::audio_player::TTSPlayer;
use crate::voice_data::{CloningStatus, Voice};
use crate::voice_persistence;
use makepad_widgets::*;
use std::path::PathBuf;

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

    // File selector row
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
            text: "Reference Audio"
        }

        file_row = <View> {
            width: Fill, height: 40
            flow: Right
            spacing: 8
            align: {y: 0.5}

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

    // Progress log area
    ProgressLog = <View> {
        width: Fill, height: 120
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

        // Modal container (centered)
        modal_container = <View> {
            width: Fill, height: Fill
            align: {x: 0.5, y: 0.5}

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
                    padding: {left: 24, right: 24, top: 20, bottom: 20}
                    spacing: 20

                    // File selector
                    file_selector = <FileSelector> {}

                    // Reference text input
                    prompt_text_input = <LabeledInput> {
                        label = { text: "Reference Text (what the audio says)" }
                        input = {
                            height: 80
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
            }
        }
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
}

impl Widget for VoiceCloneModal {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        // Initialize defaults
        if self.selected_language.is_empty() {
            self.selected_language = "zh".to_string();
        }

        let actions = match event {
            Event::Actions(actions) => actions.as_slice(),
            _ => return,
        };

        // Handle close button
        if self.view.button(ids!(modal_container.modal_content.header.close_btn)).clicked(actions) {
            self.close(cx, scope);
        }

        // Handle cancel button
        if self.view.button(ids!(modal_container.modal_content.footer.cancel_btn)).clicked(actions) {
            self.close(cx, scope);
        }

        // Handle file select button
        if self.view.button(ids!(modal_container.modal_content.body.file_selector.file_row.select_btn)).clicked(actions) {
            self.open_file_dialog(cx);
        }

        // Handle preview button
        if self.view.button(ids!(modal_container.modal_content.body.file_selector.file_row.preview_btn)).clicked(actions) {
            self.toggle_preview(cx);
        }

        // Handle language buttons
        if self.view.button(ids!(modal_container.modal_content.body.language_selector.lang_row.zh_btn)).clicked(actions) {
            self.selected_language = "zh".to_string();
            self.update_language_buttons(cx);
        }
        if self.view.button(ids!(modal_container.modal_content.body.language_selector.lang_row.en_btn)).clicked(actions) {
            self.selected_language = "en".to_string();
            self.update_language_buttons(cx);
        }

        // Handle save button
        if self.view.button(ids!(modal_container.modal_content.footer.save_btn)).clicked(actions) {
            self.save_voice(cx, scope);
        }

        // Handle overlay click to close
        let overlay = self.view.view(ids!(overlay));
        if let Hit::FingerUp(fe) = event.hits(cx, overlay.area()) {
            if !fe.is_over {
                // Click was on overlay, not content
            } else {
                // Check if click is outside modal content
                let modal_content = self.view.view(ids!(modal_container.modal_content));
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
        self.view.label(ids!(modal_container.modal_content.body.progress_log.log_scroll.log_content))
            .set_text(cx, &log_text);
        self.view.redraw(cx);
    }

    fn clear_log(&mut self, cx: &mut Cx) {
        self.log_messages.clear();
        self.view.label(ids!(modal_container.modal_content.body.progress_log.log_scroll.log_content))
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
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown");
        self.view.label(ids!(modal_container.modal_content.body.file_selector.file_row.file_name))
            .set_text(cx, file_name);

        // Validate audio file
        self.add_log(cx, "[INFO] Validating audio file...");

        match voice_persistence::validate_audio_file(&path) {
            Ok(info) => {
                self.add_log(cx, &format!(
                    "[INFO] Audio OK: {:.1}s, {}Hz, {} channels",
                    info.duration_secs, info.sample_rate, info.channels
                ));

                for warning in &info.warnings {
                    self.add_log(cx, &format!("[WARN] {}", warning));
                }

                // Update audio info label
                let info_text = format!(
                    "Duration: {:.1}s | Sample rate: {}Hz | Channels: {}",
                    info.duration_secs, info.sample_rate, info.channels
                );
                self.view.label(ids!(modal_container.modal_content.body.file_selector.audio_info))
                    .set_text(cx, &info_text);

                self.audio_info = Some(info);
                self.selected_file = Some(path);

                // Show preview button
                self.view.button(ids!(modal_container.modal_content.body.file_selector.file_row.preview_btn))
                    .set_visible(cx, true);
            }
            Err(e) => {
                self.add_log(cx, &format!("[ERROR] {}", e));
                self.selected_file = None;
                self.audio_info = None;
                self.view.button(ids!(modal_container.modal_content.body.file_selector.file_row.preview_btn))
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

        let reader = WavReader::open(path)
            .map_err(|e| format!("Failed to open WAV: {}", e))?;

        let spec = reader.spec();
        let sample_rate = spec.sample_rate;
        let channels = spec.channels as usize;

        // Read samples
        let samples: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Int => {
                let bits = spec.bits_per_sample;
                let max_val = (1 << (bits - 1)) as f32;
                reader.into_samples::<i32>()
                    .filter_map(Result::ok)
                    .map(|s| s as f32 / max_val)
                    .collect()
            }
            hound::SampleFormat::Float => {
                reader.into_samples::<f32>()
                    .filter_map(Result::ok)
                    .collect()
            }
        };

        // Convert to mono
        let mono_samples: Vec<f32> = if channels > 1 {
            samples.chunks(channels)
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
        self.view.button(ids!(modal_container.modal_content.body.file_selector.file_row.preview_btn))
            .apply_over(cx, live! {
                draw_bg: { playing: (playing_val) }
            });
        self.view.redraw(cx);
    }

    fn update_language_buttons(&mut self, cx: &mut Cx) {
        let zh_selected = if self.selected_language == "zh" { 1.0 } else { 0.0 };
        let en_selected = if self.selected_language == "en" { 1.0 } else { 0.0 };

        self.view.button(ids!(modal_container.modal_content.body.language_selector.lang_row.zh_btn))
            .apply_over(cx, live! {
                draw_bg: { selected: (zh_selected) }
                draw_text: { selected: (zh_selected) }
            });

        self.view.button(ids!(modal_container.modal_content.body.language_selector.lang_row.en_btn))
            .apply_over(cx, live! {
                draw_bg: { selected: (en_selected) }
                draw_text: { selected: (en_selected) }
            });

        self.view.redraw(cx);
    }

    fn save_voice(&mut self, cx: &mut Cx, scope: &mut Scope) {
        // Validate inputs
        let voice_name = self.view
            .text_input(ids!(modal_container.modal_content.body.voice_name_input.input))
            .text();

        if voice_name.trim().is_empty() {
            self.add_log(cx, "[ERROR] Please enter a voice name");
            return;
        }

        let prompt_text = self.view
            .text_input(ids!(modal_container.modal_content.body.prompt_text_input.input))
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
                self.add_log(cx, "[INFO] Voice created successfully!");
                self.cloning_status = CloningStatus::Completed;

                // Emit action
                cx.widget_action(
                    self.widget_uid(),
                    &scope.path,
                    VoiceCloneModalAction::VoiceCreated(voice),
                );

                // Close modal after short delay
                self.close(cx, scope);
            }
            Err(e) => {
                self.add_log(cx, &format!("[ERROR] Failed to save: {}", e));
                self.cloning_status = CloningStatus::Error(e);
            }
        }
    }

    fn close(&mut self, cx: &mut Cx, scope: &mut Scope) {
        // Stop any preview playing
        if let Some(player) = &self.preview_player {
            player.stop();
        }
        self.preview_playing = false;

        // Reset state
        self.selected_file = None;
        self.audio_info = None;
        self.cloning_status = CloningStatus::Idle;
        self.clear_log(cx);

        // Reset form fields
        self.view.text_input(ids!(modal_container.modal_content.body.voice_name_input.input))
            .set_text(cx, "");
        self.view.text_input(ids!(modal_container.modal_content.body.prompt_text_input.input))
            .set_text(cx, "");
        self.view.label(ids!(modal_container.modal_content.body.file_selector.file_row.file_name))
            .set_text(cx, "No file selected");
        self.view.label(ids!(modal_container.modal_content.body.file_selector.audio_info))
            .set_text(cx, "");
        self.view.button(ids!(modal_container.modal_content.body.file_selector.file_row.preview_btn))
            .set_visible(cx, false);

        // Hide modal
        self.view.set_visible(cx, false);

        // Emit closed action
        cx.widget_action(
            self.widget_uid(),
            &scope.path,
            VoiceCloneModalAction::Closed,
        );
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
            inner.view.view(ids!(modal_container.modal_content)).apply_over(cx, live! {
                draw_bg: { dark_mode: (dark_mode) }
            });

            // Apply to header
            inner.view.view(ids!(modal_container.modal_content.header)).apply_over(cx, live! {
                draw_bg: { dark_mode: (dark_mode) }
            });

            // Apply to footer
            inner.view.view(ids!(modal_container.modal_content.footer)).apply_over(cx, live! {
                draw_bg: { dark_mode: (dark_mode) }
            });

            inner.view.redraw(cx);
        }
    }
}
