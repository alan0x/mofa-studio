//! TTS Screen - Main TTS interface

use crate::audio_player::TTSPlayer;
use crate::dora_integration::DoraIntegration;
use crate::log_bridge;
use crate::mofa_hero::{ConnectionStatus, MofaHeroAction, MofaHeroWidgetExt};
use crate::voice_data::TTSStatus;
use crate::voice_selector::{VoiceSelectorAction, VoiceSelectorWidgetExt};
use makepad_widgets::*;
use std::path::PathBuf;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use mofa_widgets::theme::*;
    use crate::mofa_hero::MofaHero;
    use crate::voice_selector::VoiceSelector;

    // Layout constants
    SECTION_SPACING = 12.0
    PANEL_RADIUS = 6.0
    PANEL_PADDING = 14.0

    // Splitter handle for resizing panels
    Splitter = <View> {
        width: 12, height: Fill
        margin: { left: 6, right: 6 }
        align: {x: 0.5, y: 0.5}
        cursor: ColResize

        show_bg: true
        draw_bg: {
            instance dark_mode: 0.0
            instance hover: 0.0

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                // Draw subtle grip dots
                let dot_y_start = self.rect_size.y * 0.35;
                let dot_y_end = self.rect_size.y * 0.65;
                let dot_spacing = 8.0;
                let num_dots = (dot_y_end - dot_y_start) / dot_spacing;
                let color = mix(mix((SLATE_300), (SLATE_600), self.dark_mode), (PRIMARY_400), self.hover);

                // Draw vertical dots
                let y = dot_y_start;
                sdf.circle(6.0, y, 1.5);
                sdf.fill(color);
                sdf.circle(6.0, y + dot_spacing, 1.5);
                sdf.fill(color);
                sdf.circle(6.0, y + dot_spacing * 2.0, 1.5);
                sdf.fill(color);
                sdf.circle(6.0, y + dot_spacing * 3.0, 1.5);
                sdf.fill(color);
                sdf.circle(6.0, y + dot_spacing * 4.0, 1.5);
                sdf.fill(color);

                return sdf.result;
            }
        }
    }

    // Primary button style
    PrimaryButton = <Button> {
        width: Fit, height: 42
        padding: {left: 20, right: 20}

        draw_bg: {
            instance dark_mode: 0.0
            instance disabled: 0.0
            instance hover: 0.0
            instance pressed: 0.0

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 8.0);

                let base = mix((PRIMARY_500), (PRIMARY_400), self.dark_mode);
                let hover_color = mix((PRIMARY_600), (PRIMARY_300), self.dark_mode);
                let pressed_color = mix((PRIMARY_700), (PRIMARY_500), self.dark_mode);
                let disabled_color = mix((GRAY_300), (GRAY_600), self.dark_mode);

                let color = mix(base, hover_color, self.hover);
                let color = mix(color, pressed_color, self.pressed);
                let color = mix(color, disabled_color, self.disabled);

                sdf.fill(color);
                return sdf.result;
            }
        }

        draw_text: {
            text_style: <FONT_SEMIBOLD>{ font_size: 13.0 }
            fn get_color(self) -> vec4 {
                return (WHITE);
            }
        }
    }

    // Icon button (circular) for stop
    IconButton = <Button> {
        width: 36, height: 36
        padding: 0

        draw_bg: {
            instance dark_mode: 0.0
            instance hover: 0.0
            instance pressed: 0.0

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                let center = self.rect_size * 0.5;
                sdf.circle(center.x, center.y, 17.0);

                let base = mix((SLATE_100), (SLATE_700), self.dark_mode);
                let hover_color = mix((SLATE_200), (SLATE_600), self.dark_mode);
                let pressed_color = mix((SLATE_300), (SLATE_500), self.dark_mode);

                let color = mix(base, hover_color, self.hover);
                let color = mix(color, pressed_color, self.pressed);

                sdf.fill(color);
                return sdf.result;
            }
        }

        draw_text: {
            instance dark_mode: 0.0
            text_style: { font_size: 14.0 }
            fn get_color(self) -> vec4 {
                return mix((SLATE_600), (SLATE_300), self.dark_mode);
            }
        }
    }

    // Play button (larger, primary color with play/pause icon)
    PlayButton = <Button> {
        width: 52, height: 52
        padding: 0
        margin: 0

        draw_bg: {
            instance dark_mode: 0.0
            instance hover: 0.0
            instance pressed: 0.0
            instance is_playing: 0.0

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                let center = self.rect_size * 0.5;
                sdf.circle(center.x, center.y, 25.0);

                let base = mix((PRIMARY_500), (PRIMARY_400), self.dark_mode);
                let hover_color = mix((PRIMARY_600), (PRIMARY_300), self.dark_mode);
                let pressed_color = mix((PRIMARY_700), (PRIMARY_500), self.dark_mode);

                let color = mix(base, hover_color, self.hover);
                let color = mix(color, pressed_color, self.pressed);

                sdf.fill(color);

                // Draw play or pause icon
                if self.is_playing > 0.5 {
                    // Pause icon (two vertical bars)
                    sdf.rect(18.0, 16.0, 5.0, 20.0);
                    sdf.fill((WHITE));
                    sdf.rect(29.0, 16.0, 5.0, 20.0);
                    sdf.fill((WHITE));
                } else {
                    // Play icon (triangle) - slightly offset right for optical center
                    sdf.move_to(21.0, 15.0);
                    sdf.line_to(37.0, 26.0);
                    sdf.line_to(21.0, 37.0);
                    sdf.close_path();
                    sdf.fill((WHITE));
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

    // TTS Screen - main layout with bottom audio player bar
    pub TTSScreen = {{TTSScreen}} {
        width: Fill, height: Fill
        flow: Down
        spacing: 0
        padding: 0

        show_bg: true
        draw_bg: {
            instance dark_mode: 0.0
            fn pixel(self) -> vec4 {
                return mix((DARK_BG), (DARK_BG_DARK), self.dark_mode);
            }
        }

        // Main content area (fills remaining space)
        main_content = <View> {
            width: Fill, height: Fill
            flow: Right
            spacing: 0
            padding: { left: 20, right: 20, top: 16, bottom: 16 }

            // Left column - main content area (adaptive width)
            left_column = <View> {
                width: Fill, height: Fill
                flow: Down
                spacing: 12
                align: {y: 0.0}

                // System status bar (MofaHero)
                hero = <MofaHero> {
                    width: Fill
                }

                // Main content area - text input and voice selector
                content_area = <View> {
                    width: Fill, height: Fill
                    flow: Right
                    spacing: 12

                    // Text input section (fills space)
                    input_section = <RoundedView> {
                        width: Fill, height: Fill
                        flow: Down
                        show_bg: true
                        draw_bg: {
                            instance dark_mode: 0.0
                            border_radius: 6.0
                            border_size: 1.0
                            fn pixel(self) -> vec4 {
                                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                                let bg = mix((PANEL_BG), (PANEL_BG_DARK), self.dark_mode);
                                let border = mix((BORDER), (SLATE_600), self.dark_mode);
                                sdf.fill(bg);
                                sdf.stroke(border, self.border_size);
                                return sdf.result;
                            }
                        }

                        // Header with title
                        header = <View> {
                            width: Fill, height: Fit
                            padding: {left: 16, right: 16, top: 14, bottom: 14}
                            align: {x: 0.0, y: 0.5}
                            show_bg: true
                            draw_bg: {
                                instance dark_mode: 0.0
                                fn pixel(self) -> vec4 {
                                    return mix((SLATE_50), (SLATE_800), self.dark_mode);
                                }
                            }

                            title = <Label> {
                                width: Fit, height: Fit
                                draw_text: {
                                    instance dark_mode: 0.0
                                    text_style: <FONT_SEMIBOLD>{ font_size: 14.0 }
                                    fn get_color(self) -> vec4 {
                                        return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                                    }
                                }
                                text: "Text to Speech"
                            }
                        }

                        // Text input container
                        input_container = <View> {
                            width: Fill, height: Fill
                            flow: Down
                            padding: {left: 16, right: 16, top: 16, bottom: 12}

                            text_input = <TextInput> {
                                width: Fill, height: Fill
                                padding: {left: 14, right: 14, top: 12, bottom: 12}
                                empty_text: "Enter text to convert to speech..."
                                text: "你好，我是云间"
                                ascii_only: false

                                draw_bg: {
                                    instance dark_mode: 0.0
                                    border_radius: 8.0
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
                                    text_style: { font_size: 15.0, line_spacing: 1.6 }
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

                        // Bottom bar with character count and generate button
                        bottom_bar = <View> {
                            width: Fill, height: Fit
                            flow: Right
                            align: {x: 0.0, y: 0.5}
                            padding: {left: 16, right: 16, top: 4, bottom: 16}
                            spacing: 16

                            // Character count
                            char_count = <Label> {
                                width: Fit, height: Fit
                                align: {y: 0.5}
                                draw_text: {
                                    instance dark_mode: 0.0
                                    text_style: { font_size: 12.0 }
                                    fn get_color(self) -> vec4 {
                                        return mix((TEXT_TERTIARY), (TEXT_TERTIARY_DARK), self.dark_mode);
                                    }
                                }
                                text: "8 / 5,000 characters"
                            }

                            <View> { width: Fill, height: 1 }

                            // Generate button
                            generate_btn = <PrimaryButton> {
                                text: "Generate Speech"
                            }
                        }
                    }

                    // Voice selector panel (fixed width)
                    controls_panel = <View> {
                        width: 280, height: Fill
                        flow: Down

                        // Voice selector (fills available space)
                        voice_section = <RoundedView> {
                            width: Fill, height: Fill
                            flow: Down
                            show_bg: true
                            draw_bg: {
                                instance dark_mode: 0.0
                                border_radius: 6.0
                                border_size: 1.0
                                fn pixel(self) -> vec4 {
                                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                    sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                                    let bg = mix((PANEL_BG), (PANEL_BG_DARK), self.dark_mode);
                                    let border = mix((BORDER), (SLATE_600), self.dark_mode);
                                    sdf.fill(bg);
                                    sdf.stroke(border, self.border_size);
                                    return sdf.result;
                                }
                            }

                            voice_selector = <VoiceSelector> {
                                height: Fill
                            }
                        }
                    }
                }
            }

            // Splitter handle for resizing
            splitter = <Splitter> {}

            // Right Panel: System Log
            log_section = <View> {
                width: 300, height: Fill
                flow: Right
                align: {y: 0.0}

                // Toggle button column
                toggle_column = <View> {
                    width: Fit, height: Fill
                    show_bg: true
                    draw_bg: {
                        instance dark_mode: 0.0
                        fn pixel(self) -> vec4 {
                            return mix((SLATE_100), (SLATE_800), self.dark_mode);
                        }
                    }
                    align: {x: 0.5, y: 0.0}
                    padding: {left: 4, right: 4, top: 12}

                    toggle_log_btn = <Button> {
                        width: Fit, height: Fit
                        padding: {left: 6, right: 6, top: 8, bottom: 8}
                        text: ">"

                        draw_text: {
                            instance dark_mode: 0.0
                            text_style: <FONT_BOLD>{ font_size: 10.0 }
                            fn get_color(self) -> vec4 {
                                return mix((SLATE_500), (SLATE_400), self.dark_mode);
                            }
                        }
                        draw_bg: {
                            instance hover: 0.0
                            instance pressed: 0.0
                            instance dark_mode: 0.0
                            border_radius: 4.0
                            fn pixel(self) -> vec4 {
                                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                                let base = mix((SLATE_200), (SLATE_600), self.dark_mode);
                                let hover_color = mix((SLATE_300), (SLATE_500), self.dark_mode);
                                let pressed_color = mix((SLATE_400), (SLATE_400), self.dark_mode);
                                let color = mix(mix(base, hover_color, self.hover), pressed_color, self.pressed);
                                sdf.fill(color);
                                return sdf.result;
                            }
                        }
                    }
                }

                // Log content panel with border
                log_content_column = <RoundedView> {
                    width: Fill, height: Fill
                    draw_bg: {
                        instance dark_mode: 0.0
                        border_radius: 6.0
                        border_size: 1.0
                        fn pixel(self) -> vec4 {
                            let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                            sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                            let bg = mix((PANEL_BG), (PANEL_BG_DARK), self.dark_mode);
                            let border = mix((BORDER), (SLATE_600), self.dark_mode);
                            sdf.fill(bg);
                            sdf.stroke(border, self.border_size);
                            return sdf.result;
                        }
                    }
                    flow: Down

                    log_header = <View> {
                        width: Fill, height: Fit
                        flow: Down
                        show_bg: true
                        draw_bg: {
                            instance dark_mode: 0.0
                            fn pixel(self) -> vec4 {
                                return mix((SLATE_50), (SLATE_800), self.dark_mode);
                            }
                        }

                        log_title_row = <View> {
                            width: Fill, height: Fit
                            padding: {left: 14, right: 14, top: 12, bottom: 12}
                            flow: Right
                            align: {x: 0.0, y: 0.5}

                            log_title_label = <Label> {
                                text: "System Log"
                                draw_text: {
                                    instance dark_mode: 0.0
                                    text_style: <FONT_SEMIBOLD>{ font_size: 14.0 }
                                    fn get_color(self) -> vec4 {
                                        return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                                    }
                                }
                            }

                            <View> { width: Fill, height: 1 }

                            clear_log_btn = <Button> {
                                width: Fit, height: 26
                                padding: {left: 10, right: 10}
                                text: "Clear"
                                draw_bg: {
                                    instance dark_mode: 0.0
                                    instance hover: 0.0
                                    border_radius: 4.0
                                    fn pixel(self) -> vec4 {
                                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                        sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                                        let base = mix((SLATE_100), (SLATE_700), self.dark_mode);
                                        let hover_color = mix((SLATE_200), (SLATE_600), self.dark_mode);
                                        sdf.fill(mix(base, hover_color, self.hover));
                                        return sdf.result;
                                    }
                                }
                                draw_text: {
                                    instance dark_mode: 0.0
                                    text_style: { font_size: 11.0 }
                                    fn get_color(self) -> vec4 {
                                        return mix((SLATE_600), (SLATE_300), self.dark_mode);
                                    }
                                }
                            }
                        }
                    }

                    log_scroll = <ScrollYView> {
                        width: Fill, height: Fill
                        flow: Down
                        scroll_bars: <ScrollBars> {
                            show_scroll_x: false
                            show_scroll_y: true
                        }

                        log_content_wrapper = <View> {
                            width: Fill, height: Fit
                            padding: { left: 14, right: 14, top: 10, bottom: 10 }
                            flow: Down

                            log_content = <Markdown> {
                                width: Fill, height: Fit
                                font_size: 11.0
                                font_color: (GRAY_600)
                                paragraph_spacing: 6

                                draw_normal: {
                                    instance dark_mode: 0.0
                                    text_style: <FONT_REGULAR>{ font_size: 11.0 }
                                    fn get_color(self) -> vec4 {
                                        return mix((SLATE_600), (TEXT_SECONDARY_DARK), self.dark_mode);
                                    }
                                }
                                draw_bold: {
                                    instance dark_mode: 0.0
                                    text_style: <FONT_SEMIBOLD>{ font_size: 11.0 }
                                    fn get_color(self) -> vec4 {
                                        return mix((SLATE_700), (TEXT_PRIMARY_DARK), self.dark_mode);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Bottom audio player bar (like minimax)
        audio_player_bar = <View> {
            width: Fill, height: 80
            flow: Right
            align: {x: 0.5, y: 0.5}
            padding: {left: 24, right: 24, top: 12, bottom: 12}
            spacing: 0

            show_bg: true
            draw_bg: {
                instance dark_mode: 0.0
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    // Top border
                    sdf.rect(0.0, 0.0, self.rect_size.x, 1.0);
                    let border = mix((BORDER), (SLATE_700), self.dark_mode);
                    sdf.fill(border);
                    // Background
                    sdf.rect(0.0, 1.0, self.rect_size.x, self.rect_size.y - 1.0);
                    let bg = mix((SURFACE), (SURFACE_DARK), self.dark_mode);
                    sdf.fill(bg);
                    return sdf.result;
                }
            }

            // Left: Voice info (fixed width for balance)
            voice_info = <View> {
                width: 220, height: Fill
                flow: Right
                align: {x: 0.0, y: 0.5}
                spacing: 12

                // Voice avatar
                voice_avatar = <RoundedView> {
                    width: 48, height: 48
                    align: {x: 0.5, y: 0.5}
                    draw_bg: {
                        instance dark_mode: 0.0
                        border_radius: 10.0
                        fn pixel(self) -> vec4 {
                            let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                            sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                            let color = mix((PRIMARY_500), (PRIMARY_400), self.dark_mode);
                            sdf.fill(color);
                            return sdf.result;
                        }
                    }

                    avatar_initial = <Label> {
                        width: Fill, height: Fill
                        align: {x: 0.5, y: 0.5}
                        draw_text: {
                            text_style: <FONT_BOLD>{ font_size: 18.0 }
                            fn get_color(self) -> vec4 {
                                return (WHITE);
                            }
                        }
                        text: "z"
                    }
                }

                // Voice name
                voice_name_container = <View> {
                    width: Fit, height: Fit
                    flow: Down
                    align: {y: 0.5}
                    spacing: 4

                    current_voice_name = <Label> {
                        width: Fit, height: Fit
                        draw_text: {
                            instance dark_mode: 0.0
                            text_style: <FONT_SEMIBOLD>{ font_size: 14.0 }
                            fn get_color(self) -> vec4 {
                                return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                            }
                        }
                        text: "zf_xiaoni"
                    }

                    status_label = <Label> {
                        width: Fit, height: Fit
                        draw_text: {
                            instance dark_mode: 0.0
                            text_style: { font_size: 12.0 }
                            fn get_color(self) -> vec4 {
                                return mix((TEXT_TERTIARY), (TEXT_TERTIARY_DARK), self.dark_mode);
                            }
                        }
                        text: "Playing"
                    }
                }
            }

            // Center: Playback controls (fills space)
            playback_controls = <View> {
                width: Fill, height: Fill
                flow: Down
                align: {x: 0.5, y: 0.5}
                spacing: 10

                // Control buttons row - centered
                controls_row = <View> {
                    width: Fill, height: Fit
                    flow: Right
                    align: {x: 0.5, y: 0.5}
                    spacing: 12

                    // Play/Pause button
                    play_btn = <PlayButton> {
                        text: ""
                    }

                    // Stop button
                    stop_btn = <IconButton> {
                        text: "■"
                    }
                }

                // Progress bar row - centered with fixed max width
                progress_row = <View> {
                    width: Fill, height: Fit
                    flow: Right
                    align: {x: 0.5, y: 0.5}
                    spacing: 10

                    current_time = <Label> {
                        width: 50, height: Fit
                        align: {x: 1.0, y: 0.5}
                        draw_text: {
                            instance dark_mode: 0.0
                            text_style: { font_size: 12.0 }
                            fn get_color(self) -> vec4 {
                                return mix((TEXT_TERTIARY), (TEXT_TERTIARY_DARK), self.dark_mode);
                            }
                        }
                        text: "00:00"
                    }

                    // Progress bar container
                    progress_bar_container = <View> {
                        width: 400, height: 6
                        align: {y: 0.5}

                        // Progress bar
                        progress_bar = <View> {
                            width: Fill, height: Fill
                            show_bg: true
                            draw_bg: {
                                instance dark_mode: 0.0
                                instance progress: 0.0
                                fn pixel(self) -> vec4 {
                                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                    // Background track
                                    sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 3.0);
                                    let track_color = mix((GRAY_200), (GRAY_700), self.dark_mode);
                                    sdf.fill(track_color);
                                    // Progress fill
                                    let progress_width = self.rect_size.x * self.progress;
                                    sdf.box(0.0, 0.0, progress_width, self.rect_size.y, 3.0);
                                    sdf.fill((PRIMARY_500));
                                    return sdf.result;
                                }
                            }
                        }
                    }

                    total_time = <Label> {
                        width: 50, height: Fit
                        align: {x: 0.0, y: 0.5}
                        draw_text: {
                            instance dark_mode: 0.0
                            text_style: { font_size: 12.0 }
                            fn get_color(self) -> vec4 {
                                return mix((TEXT_TERTIARY), (TEXT_TERTIARY_DARK), self.dark_mode);
                            }
                        }
                        text: "00:02"
                    }
                }
            }

            // Right: Download button (fixed width for balance)
            download_section = <View> {
                width: 140, height: Fill
                align: {x: 1.0, y: 0.5}

                download_btn = <Button> {
                    width: Fit, height: 40
                    padding: {left: 24, right: 24}
                    text: "Download"

                    draw_bg: {
                        instance dark_mode: 0.0
                        instance hover: 0.0
                        instance pressed: 0.0

                        fn pixel(self) -> vec4 {
                            let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                            sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 8.0);

                            // Light blue tint background
                            let base = mix(vec4(0.23, 0.51, 0.97, 0.08), vec4(0.23, 0.51, 0.97, 0.15), self.dark_mode);
                            let hover_color = mix(vec4(0.23, 0.51, 0.97, 0.15), vec4(0.23, 0.51, 0.97, 0.25), self.dark_mode);
                            let pressed_color = mix(vec4(0.23, 0.51, 0.97, 0.25), vec4(0.23, 0.51, 0.97, 0.35), self.dark_mode);
                            let border = mix((PRIMARY_400), (PRIMARY_300), self.dark_mode);

                            let color = mix(base, hover_color, self.hover);
                            let color = mix(color, pressed_color, self.pressed);

                            sdf.fill(color);
                            sdf.stroke(border, 1.5);
                            return sdf.result;
                        }
                    }

                    draw_text: {
                        instance dark_mode: 0.0
                        text_style: <FONT_SEMIBOLD>{ font_size: 13.0 }
                        fn get_color(self) -> vec4 {
                            return mix((PRIMARY_600), (PRIMARY_300), self.dark_mode);
                        }
                    }
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct TTSScreen {
    #[deref]
    view: View,

    #[rust]
    tts_status: TTSStatus,

    #[rust]
    audio_player: Option<TTSPlayer>,

    #[rust]
    dora: Option<DoraIntegration>,

    #[rust]
    update_timer: Timer,

    #[rust]
    dark_mode: f64,

    // UI Logic states
    #[rust]
    log_panel_width: f64,
    #[rust]
    log_panel_collapsed: bool,
    #[rust]
    splitter_dragging: bool,
    #[rust]
    log_entries: Vec<String>,
    #[rust]
    logs_initialized: bool,
    #[rust]
    audio_playing_time: f64,

    // Stored audio for playback/download (not auto-play)
    #[rust]
    stored_audio_samples: Vec<f32>,
    #[rust]
    stored_audio_sample_rate: u32,

    // Current voice name for display
    #[rust]
    current_voice_name: String,
}

impl Widget for TTSScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        // Initialize audio player
        if self.audio_player.is_none() {
            self.audio_player = Some(TTSPlayer::new());
        }

        // Initialize log bridge and timer
        if !self.logs_initialized {
            log_bridge::init();
            self.logs_initialized = true;
            // Start timer for polling
            self.update_timer = cx.start_interval(0.1);
            // Initialize stored audio sample rate
            self.stored_audio_sample_rate = 24000;
            // Initialize voice name
            self.current_voice_name = "Luo Xiang".to_string();
            // Add initial log entries directly (not via log crate to ensure they show)
            self.log_entries
                .push("[INFO] [tts] MoFA TTS initialized".to_string());
            self.log_entries
                .push("[INFO] [tts] Default voice: Luo Xiang (zm_yunjian)".to_string());
            self.log_entries
                .push("[INFO] [tts] Click 'Start' to connect to MoFA bridge".to_string());
            // Update log display immediately
            self.update_log_display(cx);
        }

        // Initialize Dora (lazy, now controlled by MofaHero)
        if self.dora.is_none() {
            let dora = DoraIntegration::new();
            self.dora = Some(dora);
        }

        // Poll for audio and logs
        if self.update_timer.is_event(event).is_some() {
            // Poll Dora Audio - store audio samples instead of auto-playing
            if let Some(dora) = &self.dora {
                if dora.is_running() {
                    let shared = dora.shared_dora_state();
                    let chunks = shared.audio.drain();
                    if !chunks.is_empty() {
                        for audio in chunks {
                            self.stored_audio_samples.extend(&audio.samples);
                            self.stored_audio_sample_rate = audio.sample_rate;
                        }
                        // Transition to Ready state - user must click Play
                        if self.tts_status == TTSStatus::Generating {
                            let sample_count = self.stored_audio_samples.len();
                            let duration_secs = if self.stored_audio_sample_rate > 0 {
                                sample_count as f32 / self.stored_audio_sample_rate as f32
                            } else {
                                0.0
                            };
                            self.add_log(
                                cx,
                                &format!(
                                    "[INFO] [tts] Audio generated: {} samples, {:.1}s duration",
                                    sample_count, duration_secs
                                ),
                            );
                            self.tts_status = TTSStatus::Ready;
                            self.update_player_bar(cx);
                        }
                    }
                }
            }

            // Check if audio finished playing
            if self.tts_status == TTSStatus::Playing {
                if let Some(player) = &self.audio_player {
                    if !player.is_playing() {
                        self.audio_playing_time += 0.1;
                        if self.audio_playing_time > 0.5 {
                            self.tts_status = TTSStatus::Ready;
                            self.update_player_bar(cx);
                        }
                    }
                }
            }

            // Poll Logs from log_bridge
            let logs = log_bridge::poll_logs();
            if !logs.is_empty() {
                for log_msg in logs {
                    self.log_entries.push(log_msg.format());
                }
                self.update_log_display(cx);
            }
        }

        // Handle MofaHero Actions (Start/Stop)
        let actions = match event {
            Event::Actions(actions) => actions.as_slice(),
            _ => &[],
        };

        for action in actions {
            match action.as_widget_action().cast() {
                MofaHeroAction::StartClicked => {
                    self.start_dora(cx);
                }
                MofaHeroAction::StopClicked => {
                    self.stop_dora(cx);
                }
                MofaHeroAction::None => {}
            }

            // Handle voice selector actions
            match action.as_widget_action().cast() {
                VoiceSelectorAction::VoiceSelected(voice_id) => {
                    // Update voice name in player bar
                    if let Some(voice) = self.get_voice_by_id(&voice_id) {
                        self.current_voice_name = voice.clone();
                        self.view
                            .label(ids!(
                                audio_player_bar
                                    .voice_info
                                    .voice_name_container
                                    .current_voice_name
                            ))
                            .set_text(cx, &voice);
                        // Update avatar initial
                        let initial = voice.chars().next().unwrap_or('?').to_string();
                        self.view
                            .label(ids!(
                                audio_player_bar.voice_info.voice_avatar.avatar_initial
                            ))
                            .set_text(cx, &initial);
                    }
                    self.add_log(cx, &format!("[INFO] [tts] Voice selected: {}", voice_id));
                }
                VoiceSelectorAction::PreviewRequested(voice_id) => {
                    self.add_log(cx, &format!("[INFO] [tts] Voice preview: {}", voice_id));
                }
                VoiceSelectorAction::None => {}
            }
        }

        // Handle text input changes
        if self
            .view
            .text_input(ids!(
                main_content
                    .left_column
                    .content_area
                    .input_section
                    .input_container
                    .text_input
            ))
            .changed(actions)
            .is_some()
        {
            self.update_char_count(cx);
        }

        // Handle generate button
        if self
            .view
            .button(ids!(
                main_content
                    .left_column
                    .content_area
                    .input_section
                    .bottom_bar
                    .generate_btn
            ))
            .clicked(actions)
        {
            self.generate_speech(cx);
        }

        // Handle play button in audio player bar
        if self
            .view
            .button(ids!(
                audio_player_bar.playback_controls.controls_row.play_btn
            ))
            .clicked(actions)
        {
            self.toggle_playback(cx);
        }

        // Handle stop button in audio player bar
        if self
            .view
            .button(ids!(
                audio_player_bar.playback_controls.controls_row.stop_btn
            ))
            .clicked(actions)
        {
            self.stop_playback(cx);
        }

        // Handle download button in audio player bar
        if self
            .view
            .button(ids!(audio_player_bar.download_section.download_btn))
            .clicked(actions)
        {
            self.download_audio(cx);
        }

        // Handle clear logs
        if self
            .view
            .button(ids!(
                main_content
                    .log_section
                    .log_content_column
                    .log_header
                    .log_title_row
                    .clear_log_btn
            ))
            .clicked(actions)
        {
            self.log_entries.clear();
            self.update_log_display(cx);
        }

        // Handle toggle log panel button
        if self
            .view
            .button(ids!(main_content.log_section.toggle_column.toggle_log_btn))
            .clicked(actions)
        {
            self.toggle_log_panel(cx);
        }

        // Handle splitter
        let splitter = self.view.view(ids!(main_content.splitter));
        match event.hits(cx, splitter.area()) {
            Hit::FingerDown(_) => {
                self.splitter_dragging = true;
            }
            Hit::FingerMove(fm) => {
                if self.splitter_dragging {
                    self.resize_log_panel(cx, fm.abs.x);
                }
            }
            Hit::FingerUp(_) => {
                self.splitter_dragging = false;
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl TTSScreen {
    fn add_log(&mut self, cx: &mut Cx, message: &str) {
        self.log_entries.push(message.to_string());
        self.update_log_display(cx);
    }

    fn get_voice_by_id(&self, voice_id: &str) -> Option<String> {
        // Map voice IDs to display names
        match voice_id {
            "zm_yunjian" => Some("Luo Xiang".to_string()),
            "zf_xiaoxiao" => Some("Xiao Xiao".to_string()),
            "zf_xiaoyi" => Some("Xiao Yi".to_string()),
            "zm_yunxi" => Some("Yun Xi".to_string()),
            "zm_yunyang" => Some("Yun Yang".to_string()),
            "zf_yunxia" => Some("Yun Xia".to_string()),
            _ => Some(voice_id.to_string()),
        }
    }

    fn update_char_count(&mut self, cx: &mut Cx) {
        let text = self
            .view
            .text_input(ids!(
                main_content
                    .left_column
                    .content_area
                    .input_section
                    .input_container
                    .text_input
            ))
            .text();
        let count = text.chars().count();
        let label = format!("{} / 5,000 characters", count);
        self.view
            .label(ids!(
                main_content
                    .left_column
                    .content_area
                    .input_section
                    .bottom_bar
                    .char_count
            ))
            .set_text(cx, &label);
    }

    fn update_player_bar(&mut self, cx: &mut Cx) {
        // Update status label
        let status_text = match &self.tts_status {
            TTSStatus::Idle => "Ready",
            TTSStatus::Generating => "Generating...",
            TTSStatus::Playing => "Playing",
            TTSStatus::Ready => "Audio Ready",
            TTSStatus::Error(msg) => msg.as_str(),
        };
        self.view
            .label(ids!(
                audio_player_bar
                    .voice_info
                    .voice_name_container
                    .status_label
            ))
            .set_text(cx, status_text);

        // Update play button state
        let is_playing = self.tts_status == TTSStatus::Playing;
        self.view
            .button(ids!(
                audio_player_bar.playback_controls.controls_row.play_btn
            ))
            .apply_over(
                cx,
                live! {
                    draw_bg: { is_playing: (if is_playing { 1.0 } else { 0.0 }) }
                },
            );

        // Update total time
        if !self.stored_audio_samples.is_empty() && self.stored_audio_sample_rate > 0 {
            let duration_secs =
                self.stored_audio_samples.len() as f32 / self.stored_audio_sample_rate as f32;
            let mins = (duration_secs / 60.0) as u32;
            let secs = (duration_secs % 60.0) as u32;
            let time_str = format!("{:02}:{:02}", mins, secs);
            self.view
                .label(ids!(
                    audio_player_bar.playback_controls.progress_row.total_time
                ))
                .set_text(cx, &time_str);
        }

        self.view.redraw(cx);
    }

    fn update_log_display(&mut self, cx: &mut Cx) {
        let log_text = if self.log_entries.is_empty() {
            "*No log entries*".to_string()
        } else {
            self.log_entries
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join("\n\n")
        };

        self.view
            .markdown(ids!(
                main_content
                    .log_section
                    .log_content_column
                    .log_scroll
                    .log_content_wrapper
                    .log_content
            ))
            .set_text(cx, &log_text);
        self.view.redraw(cx);
    }

    fn toggle_log_panel(&mut self, cx: &mut Cx) {
        self.log_panel_collapsed = !self.log_panel_collapsed;

        if self.log_panel_width == 0.0 {
            self.log_panel_width = 320.0;
        }

        if self.log_panel_collapsed {
            self.view
                .view(ids!(main_content.log_section))
                .apply_over(cx, live! { width: Fit });
            self.view
                .view(ids!(main_content.log_section.log_content_column))
                .set_visible(cx, false);
            self.view
                .button(ids!(main_content.log_section.toggle_column.toggle_log_btn))
                .set_text(cx, "<");
            self.view
                .view(ids!(main_content.splitter))
                .apply_over(cx, live! { width: 0 });
        } else {
            let width = self.log_panel_width;
            self.view
                .view(ids!(main_content.log_section))
                .apply_over(cx, live! { width: (width) });
            self.view
                .view(ids!(main_content.log_section.log_content_column))
                .set_visible(cx, true);
            self.view
                .button(ids!(main_content.log_section.toggle_column.toggle_log_btn))
                .set_text(cx, ">");
            self.view
                .view(ids!(main_content.splitter))
                .apply_over(cx, live! { width: 16 });
        }

        self.view.redraw(cx);
    }

    fn resize_log_panel(&mut self, cx: &mut Cx, abs_x: f64) {
        let container_rect = self.view.area().rect(cx);
        let padding = 16.0;
        let new_log_width = (container_rect.pos.x + container_rect.size.x - abs_x - padding)
            .max(150.0)
            .min(container_rect.size.x - 400.0);

        self.log_panel_width = new_log_width;

        self.view
            .view(ids!(main_content.log_section))
            .apply_over(cx, live! { width: (new_log_width) });

        self.view.redraw(cx);
    }

    fn start_dora(&mut self, cx: &mut Cx) {
        // Check if dora exists and is not running
        let should_start = self.dora.as_ref().map(|d| !d.is_running()).unwrap_or(false);

        if !should_start {
            return;
        }

        let dataflow_path = PathBuf::from("apps/mofa-tts/dataflow/tts.yml");
        if !dataflow_path.exists() {
            self.log_entries.push(
                "[ERROR] [tts] Dataflow file not found: apps/mofa-tts/dataflow/tts.yml".to_string(),
            );
            self.update_log_display(cx);
            self.view
                .mofa_hero(ids!(main_content.left_column.hero))
                .set_connection_status(cx, ConnectionStatus::Failed);
            return;
        }

        self.log_entries
            .push("[INFO] [tts] Starting TTS dataflow...".to_string());
        self.update_log_display(cx);

        // Start dora
        if let Some(dora) = &mut self.dora {
            dora.start_dataflow(dataflow_path);
        }

        self.view
            .mofa_hero(ids!(main_content.left_column.hero))
            .set_running(cx, true);
        self.view
            .mofa_hero(ids!(main_content.left_column.hero))
            .set_connection_status(cx, ConnectionStatus::Connecting);

        self.log_entries
            .push("[INFO] [tts] Dataflow started, connecting...".to_string());
        self.update_log_display(cx);

        self.view
            .mofa_hero(ids!(main_content.left_column.hero))
            .set_connection_status(cx, ConnectionStatus::Connected);

        self.log_entries
            .push("[INFO] [tts] Connected to MoFA bridge".to_string());
        self.update_log_display(cx);
    }

    fn stop_dora(&mut self, cx: &mut Cx) {
        // Check if dora exists
        if self.dora.is_none() {
            return;
        }

        self.log_entries
            .push("[INFO] [tts] Stopping TTS dataflow...".to_string());
        self.update_log_display(cx);

        // Stop dora
        if let Some(dora) = &mut self.dora {
            dora.stop_dataflow();
        }

        self.view
            .mofa_hero(ids!(main_content.left_column.hero))
            .set_running(cx, false);
        self.view
            .mofa_hero(ids!(main_content.left_column.hero))
            .set_connection_status(cx, ConnectionStatus::Stopped);

        self.log_entries
            .push("[INFO] [tts] Dataflow stopped".to_string());
        self.update_log_display(cx);
    }

    fn generate_speech(&mut self, cx: &mut Cx) {
        // Check if Dora is connected
        let is_running = self.dora.as_ref().map(|d| d.is_running()).unwrap_or(false);
        if !is_running {
            self.add_log(
                cx,
                "[WARN] [tts] Bridge not connected. Please start MoFA first.",
            );
            return;
        }

        let text = self
            .view
            .text_input(ids!(
                main_content
                    .left_column
                    .content_area
                    .input_section
                    .input_container
                    .text_input
            ))
            .text();
        if text.is_empty() {
            self.add_log(
                cx,
                "[WARN] [tts] Please enter some text to convert to speech.",
            );
            return;
        }

        let log_text = if text.len() > 50 {
            format!("{}...", &text[..50])
        } else {
            text.clone()
        };
        self.add_log(
            cx,
            &format!("[INFO] [tts] Generating speech for: '{}'", log_text),
        );

        let voice_id = self
            .view
            .voice_selector(ids!(
                main_content
                    .left_column
                    .content_area
                    .controls_panel
                    .voice_section
                    .voice_selector
            ))
            .selected_voice_id()
            .unwrap_or_else(|| "zm_yunjian".to_string());

        self.add_log(cx, &format!("[INFO] [tts] Using voice: {}", voice_id));

        // Clear previous audio
        self.stored_audio_samples.clear();
        self.stored_audio_sample_rate = 24000;

        self.tts_status = TTSStatus::Generating;
        self.update_player_bar(cx);

        let prompt = format!("VOICE:{}|{}", voice_id, text);

        // Send prompt to dora
        let send_result = self
            .dora
            .as_ref()
            .map(|d| d.send_prompt(&prompt))
            .unwrap_or(false);

        if send_result {
            self.add_log(cx, "[INFO] [tts] Prompt sent to TTS engine");
        } else {
            self.add_log(cx, "[ERROR] [tts] Failed to send prompt to Dora");
            self.tts_status = TTSStatus::Error("Failed to send prompt".to_string());
            self.update_player_bar(cx);
        }

        if let Some(player) = &self.audio_player {
            player.stop();
        }
    }

    fn toggle_playback(&mut self, cx: &mut Cx) {
        if self.tts_status == TTSStatus::Playing {
            // Pause
            if let Some(player) = &self.audio_player {
                player.pause();
            }
            self.tts_status = TTSStatus::Ready;
            self.add_log(cx, "[INFO] [tts] Playback paused");
        } else if !self.stored_audio_samples.is_empty() {
            // Play
            if let Some(player) = &self.audio_player {
                player.write_audio(&self.stored_audio_samples);
            }
            self.tts_status = TTSStatus::Playing;
            self.audio_playing_time = 0.0;
            self.add_log(cx, "[INFO] [tts] Playing audio...");
        } else {
            self.add_log(cx, "[WARN] [tts] No audio to play");
        }
        self.update_player_bar(cx);
    }

    fn stop_playback(&mut self, cx: &mut Cx) {
        if let Some(player) = &self.audio_player {
            player.stop();
        }
        if self.tts_status == TTSStatus::Playing {
            self.tts_status = TTSStatus::Ready;
            self.add_log(cx, "[INFO] [tts] Playback stopped");
        }
        // Reset progress
        self.view
            .label(ids!(
                audio_player_bar.playback_controls.progress_row.current_time
            ))
            .set_text(cx, "00:00");
        self.update_player_bar(cx);
    }

    fn download_audio(&mut self, cx: &mut Cx) {
        if self.stored_audio_samples.is_empty() {
            self.add_log(cx, "[WARN] [tts] No audio to download");
            return;
        }

        // Generate filename with timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let filename = format!("tts_output_{}.wav", timestamp);

        // Get downloads folder or current directory
        let download_path = if let Some(home) = dirs::home_dir() {
            let downloads = home.join("Downloads");
            if downloads.exists() {
                downloads.join(&filename)
            } else {
                PathBuf::from(&filename)
            }
        } else {
            PathBuf::from(&filename)
        };

        // Write WAV file
        match self.write_wav_file(&download_path) {
            Ok(_) => {
                self.add_log(
                    cx,
                    &format!("[INFO] [tts] Audio saved to: {}", download_path.display()),
                );
            }
            Err(e) => {
                self.add_log(cx, &format!("[ERROR] [tts] Failed to save audio: {}", e));
            }
        }
    }

    fn write_wav_file(&self, path: &PathBuf) -> std::io::Result<()> {
        use std::io::Write;

        let sample_rate = self.stored_audio_sample_rate;
        let num_channels: u16 = 1;
        let bits_per_sample: u16 = 16;
        let byte_rate = sample_rate * (num_channels as u32) * (bits_per_sample as u32) / 8;
        let block_align: u16 = num_channels * bits_per_sample / 8;
        let data_size = (self.stored_audio_samples.len() * 2) as u32;
        let file_size = 36 + data_size;

        let mut file = std::fs::File::create(path)?;

        // RIFF header
        file.write_all(b"RIFF")?;
        file.write_all(&file_size.to_le_bytes())?;
        file.write_all(b"WAVE")?;

        // fmt chunk
        file.write_all(b"fmt ")?;
        file.write_all(&16u32.to_le_bytes())?;
        file.write_all(&1u16.to_le_bytes())?;
        file.write_all(&num_channels.to_le_bytes())?;
        file.write_all(&sample_rate.to_le_bytes())?;
        file.write_all(&byte_rate.to_le_bytes())?;
        file.write_all(&block_align.to_le_bytes())?;
        file.write_all(&bits_per_sample.to_le_bytes())?;

        // data chunk
        file.write_all(b"data")?;
        file.write_all(&data_size.to_le_bytes())?;

        // Convert f32 samples to i16 and write
        for &sample in &self.stored_audio_samples {
            let clamped = sample.max(-1.0).min(1.0);
            let i16_sample = (clamped * 32767.0) as i16;
            file.write_all(&i16_sample.to_le_bytes())?;
        }

        Ok(())
    }
}

impl TTSScreenRef {
    pub fn update_dark_mode(&self, cx: &mut Cx, dark_mode: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.dark_mode = dark_mode;
            inner
                .view
                .apply_over(cx, live! { draw_bg: { dark_mode: (dark_mode) } });

            // Apply dark mode to MofaHero
            inner
                .view
                .mofa_hero(ids!(main_content.left_column.hero))
                .apply_over(cx, live! { draw_bg: { dark_mode: (dark_mode) } });

            // Apply dark mode to voice selector
            inner
                .view
                .voice_selector(ids!(
                    main_content
                        .left_column
                        .content_area
                        .controls_panel
                        .voice_section
                        .voice_selector
                ))
                .update_dark_mode(cx, dark_mode);

            // Apply dark mode to log markdown
            let log_markdown = inner.view.markdown(ids!(
                main_content
                    .log_section
                    .log_content_column
                    .log_scroll
                    .log_content_wrapper
                    .log_content
            ));
            log_markdown.apply_over(
                cx,
                live! {
                    draw_normal: { dark_mode: (dark_mode) }
                    draw_bold: { dark_mode: (dark_mode) }
                },
            );

            // Apply dark mode to audio player bar
            inner
                .view
                .view(ids!(audio_player_bar))
                .apply_over(cx, live! { draw_bg: { dark_mode: (dark_mode) } });

            inner.view.redraw(cx);
        }
    }
}
