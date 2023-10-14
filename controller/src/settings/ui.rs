use std::{
    borrow::Cow,
    cell::RefCell,
    rc::Rc,
    sync::atomic::Ordering,
    time::Instant,
};

use imgui::Condition;
use obfstr::obfstr;

use crate::{
    settings::{
        AppSettings,
        EspBoxType,
        LineStartPosition,
    },
    utils::ImGuiKey,
    Application,
};

pub struct SettingsUI {
    settings: Rc<RefCell<AppSettings>>,
    discord_link_copied: Option<Instant>,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
impl SettingsUI {
    pub fn new(settings: Rc<RefCell<AppSettings>>) -> Self {
        Self {
            settings,
            discord_link_copied: None,
        }
    }

    pub fn render(&mut self, app: &Application, ui: &imgui::Ui) {
        let content_font = ui.current_font().id();
        let _title_font = ui.push_font(app.fonts.valthrun);
        ui.window(obfstr!("Valthrun"))
            .size([600.0, 300.0], Condition::FirstUseEver)
            .build(|| {
                let _content_font = ui.push_font(content_font);
                let mut settings: std::cell::RefMut<'_, AppSettings> = self.settings.borrow_mut();
                if let Some(_tab_bar) = ui.tab_bar("main") {
                    if let Some(_tab) = ui.tab_item("Information") {
                        ui.text(obfstr!("Valthrun an open source CS2 external read only kernel gameplay enhancer."));
                        ui.text(&format!("{} Version {}", obfstr!("Valthrun"), VERSION));
                        ui.text(&format!("{} Version {} ({})", obfstr!("CS2"), app.cs2_build_info.revision, app.cs2_build_info.build_datetime));

                        let ydummy = ui.window_size()[1] - ui.cursor_pos()[1] - ui.text_line_height_with_spacing() * 2.0 - 12.0;
                        ui.dummy([ 0.0, ydummy ]);
                        ui.separator();

                        ui.text(obfstr!("Join our discord:"));
                        ui.text_colored([ 0.18, 0.51, 0.97, 1.0 ], obfstr!("https://discord.gg/ecKbpAPW5T"));
                        if ui.is_item_hovered() {
                            ui.set_mouse_cursor(Some(imgui::MouseCursor::Hand));
                        }

                        if ui.is_item_clicked() {
                            self.discord_link_copied = Some(Instant::now());
                            ui.set_clipboard_text(obfstr!("https://discord.gg/ecKbpAPW5T"));
                        }

                        let show_copied = self.discord_link_copied.as_ref()
                            .map(|time| time.elapsed().as_millis() < 3_000)
                            .unwrap_or(false);

                        if show_copied {
                            ui.same_line();
                            ui.text("(Copied)");
                        }
                    }

                    if let Some(_) = ui.tab_item("Hotkeys") {
                        ui.button_key(obfstr!("Toggle Settings UI"), &mut settings.key_settings, [ 150.0, 0.0 ]);
                        ui.separator();
                        ui.button_key_optional(obfstr!("ESP toggle"), &mut settings.esp_toogle, [ 150.0, 0.0 ]);
                    }

                    if let Some(_tab) = ui.tab_item("Visuals") {
                        ui.checkbox(obfstr!("ESP"), &mut settings.esp);

                        if settings.esp {
                            ui.checkbox(obfstr!("ESP Boxes"), &mut settings.esp_boxes);
                            if settings.esp_boxes {
                                ui.set_next_item_width(120.0);
                                const ESP_BOX_TYPES: [ EspBoxType; 2 ] = [ EspBoxType::Box2D, EspBoxType::Box3D ];

                                fn esp_box_type_name(value: &EspBoxType) -> Cow<'_, str> {
                                    match value {
                                        EspBoxType::Box2D => "2D",
                                        EspBoxType::Box3D => "3D",
                                    }.into()
                                }

                                let mut type_index = ESP_BOX_TYPES.iter().position(|v| *v == settings.esp_box_type).unwrap_or_default();
                                if ui.combo(obfstr!("Type"), &mut type_index, &ESP_BOX_TYPES, &esp_box_type_name) {
                                    settings.esp_box_type = ESP_BOX_TYPES[type_index];
                                }

                                ui.same_line();
                                ui.slider_config(obfstr!("Thickness"), 0.1, 10.0)
                                    .build(&mut settings.esp_boxes_thickness);
                            }
                            if settings.esp_box_type == EspBoxType::Box2D {
                                ui.checkbox(obfstr!("2DBOX: Show Health Bar"), &mut settings.esp_health_bar);
                                if settings.esp_health_bar {
                                    ui.same_line();
                                    ui.slider("Bar Width", 2.0, 20.0, &mut settings.esp_health_bar_size);
                                    ui.checkbox(obfstr!("Rainbow Health Bar (Random colors!)"), &mut settings.esp_health_bar_rainbow);
                                }
                            }

                            ui.checkbox(obfstr!("ESP Skeletons"), &mut settings.esp_skeleton);
                            if settings.esp_skeleton {
                                ui.slider_config(obfstr!("Skeleton Thickness"), 0.1, 10.0)
                                    .build(&mut settings.esp_skeleton_thickness);
                            }

                            ui.checkbox(obfstr!("Show player health"), &mut settings.esp_info_health);
                            ui.checkbox(obfstr!("Show player weapon"), &mut settings.esp_info_weapon);
                            ui.checkbox(obfstr!("Display if player has kit"), &mut settings.esp_info_kit);
                            ui.checkbox(obfstr!("Show lines"), &mut settings.esp_lines);
                            if settings.esp_lines {
                                ui.set_next_item_width(120.0);
                                const LINE_START_POSITIONS: [LineStartPosition; 7] = [
                                    LineStartPosition::TopLeft,
                                    LineStartPosition::TopCenter,
                                    LineStartPosition::TopRight,
                                    LineStartPosition::Center,
                                    LineStartPosition::BottomLeft,
                                    LineStartPosition::BottomCenter,
                                    LineStartPosition::BottomRight,
                                ];
                                fn line_start_position_name(value: &LineStartPosition) -> Cow<'_, str> {
                                    match value {
                                        LineStartPosition::TopLeft => "Top Left".into(),
                                        LineStartPosition::TopCenter => "Top Center".into(),
                                        LineStartPosition::TopRight => "Top Right".into(),
                                        LineStartPosition::Center => "Center".into(),
                                        LineStartPosition::BottomLeft => "Bottom Left".into(),
                                        LineStartPosition::BottomCenter => "Bottom Center".into(),
                                        LineStartPosition::BottomRight => "Bottom Right".into(),
                                    }
                                }
                                let mut line_position_index = LINE_START_POSITIONS.iter().position(|v| *v == settings.esp_lines_position).unwrap_or_default();
                                if ui.combo(obfstr!("Start Position"), &mut line_position_index, &LINE_START_POSITIONS, &line_start_position_name) {
                                    settings.esp_lines_position = LINE_START_POSITIONS[line_position_index];
                                }
                            }

                            ui.checkbox(obfstr!("ESP Team"), &mut settings.esp_enabled_team);
                            if settings.esp_enabled_team {
                                ui.same_line();
                                ui.color_edit4_config(obfstr!("Team Color"), &mut settings.esp_color_team)
                                    .alpha_bar(true)
                                    .inputs(false)
                                    .label(false)
                                    .build();
                                ui.same_line();
                                ui.text(obfstr!("Team Color"));
                            }

                            ui.checkbox(obfstr!("ESP Enemy"), &mut settings.esp_enabled_enemy);
                            if settings.esp_enabled_enemy {
                                ui.same_line();
                                ui.color_edit4_config(obfstr!("Enemy Color"), &mut settings.esp_color_enemy)
                                    .alpha_bar(true)
                                    .inputs(false)
                                    .label(false)
                                    .build();
                                ui.same_line();
                                ui.text(obfstr!("Enemy Color"));
                            }
                            ui.separator();
                        }

                        ui.checkbox(obfstr!("Bomb Timer"), &mut settings.bomb_timer);
                        ui.checkbox(obfstr!("Bomb Background"), &mut settings.bomb_timer_decor);
                        ui.checkbox(obfstr!("Bomb Colors"), &mut settings.bomb_timer_color);

                        ui.separator();
                        ui.checkbox(obfstr!("Spectators List"), &mut settings.spectators_list);
                    }

                    if let Some(_) = ui.tab_item(obfstr!("Aim Assist")) {
                        ui.button_key_optional(obfstr!("Trigger Bot"), &mut settings.key_trigger_bot, [150.0, 0.0]);
                        if settings.key_trigger_bot.is_some() {
                            let mut values_updated = false;

                            ui.text(obfstr!("Trigger delay: ")); ui.same_line();

                            let slider_width = (ui.current_column_width() / 2.0 - 20.0).min(300.0).max(50.0);
                            ui.set_next_item_width(slider_width);
                            values_updated |= ui.slider_config("##delay_min", 0, 250).display_format("%dms").build(&mut settings.trigger_bot_delay_min); ui.same_line();
                            ui.text(" - "); ui.same_line();
                            ui.set_next_item_width(slider_width);
                            values_updated |= ui.slider_config("##delay_max", 0, 250).display_format("%dms").build(&mut settings.trigger_bot_delay_max); 

                            if values_updated {
                                /* fixup min/max */
                                let delay_min = settings.trigger_bot_delay_min.min(settings.trigger_bot_delay_max);
                                let delay_max = settings.trigger_bot_delay_min.max(settings.trigger_bot_delay_max);

                                settings.trigger_bot_delay_min = delay_min;
                                settings.trigger_bot_delay_max = delay_max;
                            }

                            ui.checkbox(obfstr!("Retest trigger target after delay"), &mut settings.trigger_bot_check_target_after_delay);
                            ui.checkbox(obfstr!("Team Check"), &mut settings.trigger_bot_team_check);
                            ui.separator();
                        }

                        //ui.checkbox("Simle Recoil Helper", &mut settings.aim_assist_recoil);
                    }


                    if let Some(_) = ui.tab_item("Misc") {
                        ui.checkbox(obfstr!("Valthrun Watermark"), &mut settings.valthrun_watermark);

                        if ui.checkbox(obfstr!("Hide overlay from screen capture"), &mut settings.hide_overlay_from_screen_capture) {
                            app.settings_screen_capture_changed.store(true, Ordering::Relaxed);
                        }

                        if ui.checkbox(obfstr!("Show render debug overlay"), &mut settings.render_debug_window) {
                            app.settings_render_debug_window_changed.store(true, Ordering::Relaxed);
                        }
                    }
                }
            });
    }
}
