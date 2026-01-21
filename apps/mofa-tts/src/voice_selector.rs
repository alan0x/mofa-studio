//! Voice selector component - displays list of available voices

use crate::voice_data::{get_builtin_voices, Voice};
use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use mofa_widgets::theme::*;

    // Voice item in the list
    VoiceItem = <View> {
        width: Fill, height: Fit
        padding: {left: 12, right: 16, top: 10, bottom: 10}
        flow: Right
        align: {y: 0.5}
        spacing: 12
        cursor: Hand

        show_bg: true
        draw_bg: {
            instance dark_mode: 0.0
            instance selected: 0.0
            instance hover: 0.0

            fn pixel(self) -> vec4 {
                let base = mix((SURFACE), (SURFACE_DARK), self.dark_mode);
                let selected_color = mix((PRIMARY_50), (PRIMARY_900), self.dark_mode);
                let hover_color = mix((SLATE_50), (SLATE_800), self.dark_mode);

                let color = mix(base, selected_color, self.selected);
                let color = mix(color, hover_color, self.hover * (1.0 - self.selected));
                return color;
            }
        }

        // Selection indicator - left edge bar
        selection_indicator = <View> {
            width: 3, height: 36
            show_bg: true
            draw_bg: {
                instance selected: 0.0
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 1.5);
                    let color = mix(vec4(0.0, 0.0, 0.0, 0.0), (PRIMARY_500), self.selected);
                    sdf.fill(color);
                    return sdf.result;
                }
            }
        }

        // Voice avatar - circular with initial
        avatar = <RoundedView> {
            width: 36, height: 36
            align: {x: 0.5, y: 0.5}
            draw_bg: {
                instance dark_mode: 0.0
                instance selected: 0.0
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.circle(18.0, 18.0, 18.0);
                    let base_color = mix((PRIMARY_500), (PRIMARY_400), self.dark_mode);
                    let selected_color = mix((PRIMARY_600), (PRIMARY_300), self.dark_mode);
                    let color = mix(base_color, selected_color, self.selected);
                    sdf.fill(color);
                    return sdf.result;
                }
            }

            // Initial letter
            initial = <Label> {
                width: Fill, height: Fill
                align: {x: 0.3, y: 0.6}
                draw_text: {
                    text_style: <FONT_SEMIBOLD>{ font_size: 14.0 }
                    fn get_color(self) -> vec4 {
                        return (WHITE);
                    }
                }
                text: "L"
            }
        }

        // Voice info - name and description
        info = <View> {
            width: Fill, height: Fit
            flow: Down
            spacing: 2

            name = <Label> {
                width: Fill, height: Fit
                draw_text: {
                    instance dark_mode: 0.0
                    instance selected: 0.0
                    text_style: <FONT_SEMIBOLD>{ font_size: 13.0 }
                    fn get_color(self) -> vec4 {
                        let base = mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                        let selected_color = mix((PRIMARY_600), (PRIMARY_300), self.dark_mode);
                        return mix(base, selected_color, self.selected);
                    }
                }
                text: "Voice Name"
            }

            description = <Label> {
                width: Fill, height: Fit
                draw_text: {
                    instance dark_mode: 0.0
                    text_style: { font_size: 11.0 }
                    fn get_color(self) -> vec4 {
                        return mix((TEXT_TERTIARY), (TEXT_TERTIARY_DARK), self.dark_mode);
                    }
                }
                text: "Voice description"
            }
        }

        // Preview button (hidden for now - can be enabled later)
        preview_btn = <View> {
            width: 32, height: 32
            align: {x: 0.5, y: 0.5}
            cursor: Hand
            visible: false

            show_bg: true
            draw_bg: {
                instance dark_mode: 0.0
                instance hover: 0.0

                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.circle(16.0, 16.0, 16.0);
                    let base = mix((SURFACE_HOVER), (SURFACE_HOVER_DARK), self.dark_mode);
                    let hover_color = mix((PRIMARY_100), (PRIMARY_800), self.dark_mode);
                    sdf.fill(mix(base, hover_color, self.hover));
                    return sdf.result;
                }
            }

            // Play icon (triangle)
            <View> {
                width: Fill, height: Fill
                align: {x: 0.5, y: 0.5}
                show_bg: true
                draw_bg: {
                    instance dark_mode: 0.0
                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        // Draw play triangle
                        sdf.move_to(10.0, 8.0);
                        sdf.line_to(22.0, 16.0);
                        sdf.line_to(10.0, 24.0);
                        sdf.close_path();
                        let color = mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                        sdf.fill(color);
                        return sdf.result;
                    }
                }
            }
        }
    }

    // Voice selector panel
    pub VoiceSelector = {{VoiceSelector}} {
        width: Fill, height: Fill
        flow: Down

        show_bg: true
        draw_bg: {
            instance dark_mode: 0.0
            fn pixel(self) -> vec4 {
                return mix((SURFACE), (SURFACE_DARK), self.dark_mode);
            }
        }

        // Header with title and selected voice indicator
        header = <View> {
            width: Fill, height: Fit
            padding: {left: 16, right: 16, top: 14, bottom: 14}
            flow: Down
            spacing: 0
            show_bg: true
            draw_bg: {
                instance dark_mode: 0.0
                fn pixel(self) -> vec4 {
                    return mix((SLATE_50), (SLATE_800), self.dark_mode);
                }
            }

            title_row = <View> {
                width: Fill, height: Fit
                flow: Right
                align: {x: 0.0, y: 0.5}
                spacing: 8

                title = <Label> {
                    width: Fit, height: Fit
                    draw_text: {
                        instance dark_mode: 0.0
                        text_style: <FONT_SEMIBOLD>{ font_size: 14.0 }
                        fn get_color(self) -> vec4 {
                            return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                        }
                    }
                    text: "Select Voice"
                }

                <View> { width: Fill, height: 1 }

                // Selected voice badge
                selected_voice_badge = <RoundedView> {
                    width: Fit, height: Fit
                    padding: {left: 8, right: 8, top: 4, bottom: 4}
                    draw_bg: {
                        instance dark_mode: 0.0
                        border_radius: 4.0
                        fn pixel(self) -> vec4 {
                            let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                            sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                            let bg = mix((PRIMARY_100), (PRIMARY_800), self.dark_mode);
                            sdf.fill(bg);
                            return sdf.result;
                        }
                    }

                    selected_voice_label = <Label> {
                        width: Fit, height: Fit
                        draw_text: {
                            instance dark_mode: 0.0
                            text_style: <FONT_SEMIBOLD>{ font_size: 11.0 }
                            fn get_color(self) -> vec4 {
                                return mix((PRIMARY_700), (PRIMARY_200), self.dark_mode);
                            }
                        }
                        text: "小妮 (Xiaoni)"
                    }
                }
            }
        }

        // Divider
        <View> {
            width: Fill, height: 1
            show_bg: true
            draw_bg: {
                instance dark_mode: 0.0
                fn pixel(self) -> vec4 {
                    return mix((BORDER), (BORDER_DARK), self.dark_mode);
                }
            }
        }

        // Voice list with scrolling
        voice_list = <PortalList> {
            width: Fill, height: Fill
            flow: Down

            VoiceItem = <VoiceItem> {}
        }
    }
}

/// Action emitted when a voice is selected
#[derive(Clone, Debug, DefaultNone)]
pub enum VoiceSelectorAction {
    None,
    VoiceSelected(String), // voice_id
    PreviewRequested(String), // voice_id
}

#[derive(Live, LiveHook, Widget)]
pub struct VoiceSelector {
    #[deref]
    view: View,

    #[rust]
    voices: Vec<Voice>,

    #[rust]
    selected_voice_id: Option<String>,

    #[rust]
    dark_mode: f64,

    #[rust]
    initialized: bool,
}

impl Widget for VoiceSelector {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        // Initialize voices on first event
        if !self.initialized {
            self.voices = get_builtin_voices();
            // Select first voice by default
            if let Some(first) = self.voices.first() {
                self.selected_voice_id = Some(first.id.clone());
            }
            self.initialized = true;
        }

        // Extract actions from event
        let actions = match event {
            Event::Actions(actions) => actions.as_slice(),
            _ => return,
        };

        // Handle portal list item events using items_with_actions pattern
        let portal_list = self.view.portal_list(ids!(voice_list));
        for (item_id, item) in portal_list.items_with_actions(actions) {
            // Handle click on voice item
            if item.as_view().finger_up(actions).is_some() {
                if item_id < self.voices.len() {
                    let voice_id = self.voices[item_id].id.clone();
                    let voice_name = self.voices[item_id].name.clone();
                    self.selected_voice_id = Some(voice_id.clone());

                    // Update selected voice label in header badge
                    self.view.label(ids!(header.title_row.selected_voice_badge.selected_voice_label)).set_text(cx, &voice_name);

                    cx.widget_action(
                        self.widget_uid(),
                        &scope.path,
                        VoiceSelectorAction::VoiceSelected(voice_id),
                    );
                    self.view.redraw(cx);
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Initialize if needed (in case draw happens before handle_event)
        if !self.initialized {
            self.voices = get_builtin_voices();
            if let Some(first) = self.voices.first() {
                self.selected_voice_id = Some(first.id.clone());
            }
            self.initialized = true;
        }

        // Draw portal list items using borrow pattern
        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, self.voices.len());

                while let Some(item_id) = list.next_visible_item(cx) {
                    if item_id < self.voices.len() {
                        let voice = &self.voices[item_id];
                        let item = list.item(cx, item_id, live_id!(VoiceItem));

                        // Set voice data
                        let initial = voice.name.chars().next().unwrap_or('?').to_string();
                        item.label(ids!(avatar.initial)).set_text(cx, &initial);
                        item.label(ids!(info.name)).set_text(cx, &voice.name);
                        item.label(ids!(info.description)).set_text(cx, &voice.description);

                        // Set selection state
                        let is_selected = self.selected_voice_id.as_ref() == Some(&voice.id);
                        let selected_val = if is_selected { 1.0 } else { 0.0 };

                        // Apply selection and dark mode to item background
                        item.apply_over(cx, live! {
                            draw_bg: { selected: (selected_val), dark_mode: (self.dark_mode) }
                        });

                        // Apply selection indicator
                        item.view(ids!(selection_indicator)).apply_over(cx, live! {
                            draw_bg: { selected: (selected_val) }
                        });

                        // Apply dark mode and selection to avatar
                        item.view(ids!(avatar)).apply_over(cx, live! {
                            draw_bg: { dark_mode: (self.dark_mode), selected: (selected_val) }
                        });

                        // Apply dark mode and selection to name label
                        item.label(ids!(info.name)).apply_over(cx, live! {
                            draw_text: { dark_mode: (self.dark_mode), selected: (selected_val) }
                        });

                        // Apply dark mode to description
                        item.label(ids!(info.description)).apply_over(cx, live! {
                            draw_text: { dark_mode: (self.dark_mode) }
                        });

                        item.draw_all(cx, scope);
                    }
                }
            }
        }
        DrawStep::done()
    }
}

impl VoiceSelectorRef {
    /// Get currently selected voice
    pub fn selected_voice(&self) -> Option<Voice> {
        if let Some(inner) = self.borrow() {
            if let Some(voice_id) = &inner.selected_voice_id {
                return inner.voices.iter().find(|v| &v.id == voice_id).cloned();
            }
        }
        None
    }

    /// Get selected voice ID
    pub fn selected_voice_id(&self) -> Option<String> {
        self.borrow().and_then(|inner| inner.selected_voice_id.clone())
    }

    /// Update dark mode
    pub fn update_dark_mode(&self, cx: &mut Cx, dark_mode: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.dark_mode = dark_mode;

            inner.view.apply_over(cx, live! {
                draw_bg: { dark_mode: (dark_mode) }
            });

            // Header background
            inner.view.view(ids!(header)).apply_over(cx, live! {
                draw_bg: { dark_mode: (dark_mode) }
            });

            // Header title
            inner.view.label(ids!(header.title_row.title)).apply_over(cx, live! {
                draw_text: { dark_mode: (dark_mode) }
            });

            // Selected voice badge
            inner.view.view(ids!(header.title_row.selected_voice_badge)).apply_over(cx, live! {
                draw_bg: { dark_mode: (dark_mode) }
            });

            // Selected voice label
            inner.view.label(ids!(header.title_row.selected_voice_badge.selected_voice_label)).apply_over(cx, live! {
                draw_text: { dark_mode: (dark_mode) }
            });

            inner.view.redraw(cx);
        }
    }
}
