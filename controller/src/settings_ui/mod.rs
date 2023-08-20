use std::{rc::Rc, cell::RefCell, time::Instant};

use imgui::Condition;
use obfstr::obfstr;

use crate::{settings::{AppSettings, HotKey}, Application};

pub trait ImGuiKey {
    fn button_key(&self, label: &str, key: &mut HotKey, size: [f32; 2]) -> bool;
    fn button_key_optional(&self, label: &str, key: &mut Option<HotKey>, size: [f32; 2]) -> bool;
}

mod hotkey {
    use imgui::Key;

    use crate::settings::HotKey;

    pub fn render_button_key(ui: &imgui::Ui, label: &str, key: &mut Option<HotKey>, size: [f32; 2], optional: bool) -> bool {
        let _container = ui.push_id(label);

        let button_label = if let Some(key) = &key {
            format!("{:?}", key.0)
        } else {
            "None".to_string()
        };

        if !label.starts_with("##") {
            ui.text(label);
            ui.same_line();
        }

        let mut updated = false;
        if optional {
            if ui.button_with_size(&button_label, [ size[0] - 35.0, size[1] ]) {
                ui.open_popup(label);
            }

            ui.same_line_with_spacing(0.0, 10.0);

            ui.disabled(key.is_none(), || {
                if ui.button_with_size("X", [ 25.0, 0.0 ]) {
                    updated = true;
                    *key = None;
                }
            });
        } else {
            if ui.button_with_size(&button_label, size) {
                ui.open_popup(label);
            }
        }

        ui.modal_popup_config(label)
            .inputs(true)
            .collapsible(true)
            .movable(false)
            .menu_bar(false)
            .resizable(false)
            .title_bar(false)
            .build(|| {
                ui.text("Press any key or ESC to exit");

                if ui.is_key_pressed(Key::Escape) {
                    ui.close_current_popup();
                } else {
                    for key_variant in Key::VARIANTS {
                        if ui.is_key_pressed(key_variant) {
                            *key = Some(HotKey(key_variant));
                            updated = true;
                            ui.close_current_popup();
                        }
                    }
                }
            });

        updated
    }
}

impl ImGuiKey for imgui::Ui {
    fn button_key(&self, label: &str, key: &mut HotKey, size: [f32; 2]) -> bool {
        let mut key_opt = Some(key.clone());
        if hotkey::render_button_key(self, label, &mut key_opt, size, false) {
            *key = key_opt.unwrap();
            true
        } else {
            false
        }
    }

    fn button_key_optional(&self, label: &str, key: &mut Option<HotKey>, size: [f32; 2]) -> bool {
        hotkey::render_button_key(self, label, key, size, true)
    }
}

pub struct SettingsUI {
    settings: Rc<RefCell<AppSettings>>,
    discord_link_copied: Option<Instant>
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
        ui.window(obfstr!("Valthrun"))
            .size([600.0, 300.0], Condition::FirstUseEver)
            .build(|| {
                let mut settings = self.settings.borrow_mut();
                if let Some(_tab_bar) = ui.tab_bar("main") {
                    if let Some(_tab) = ui.tab_item("Information") {
                        ui.text(obfstr!("Valthrun an open source CS2 external read only kernel gameplay enhancer."));
                        ui.text(&format!("{} Version {}", obfstr!("Valthrun"), VERSION));
                        ui.text(&format!("{} Version {} ({})", obfstr!("CS2"), app.cs2_build_info.revision, app.cs2_build_info.build_datetime));
                        
                        let ydummy = ui.window_size()[1] - ui.cursor_pos()[1] - ui.text_line_height_with_spacing() * 2.5;
                        ui.dummy([ 0.0, ydummy ]);
                        ui.separator();

                        ui.text("Join our discord:");
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
                        ui.button_key("Toggle Settings", &mut settings.key_settings, [150.0, 0.0]);
                    }

                    if let Some(_tab) = ui.tab_item("Visuals") {
                        ui.checkbox(obfstr!("ESP Boxes"), &mut settings.esp_boxes);
                        ui.slider_config("Box Thickness", 0.1, 10.0)
                            .build(&mut settings.esp_boxes_thickness);
                        ui.checkbox(obfstr!("ESP Skeletons"), &mut settings.esp_skeleton);
                        ui.slider_config("Skeleton Thickness", 0.1, 10.0)
                            .build(&mut settings.esp_skeleton_thickness);
                        ui.checkbox(obfstr!("Bomb Timer"), &mut settings.bomb_timer);

                        ui.color_edit4_config("Team Color", &mut settings.esp_color_team)
                            .alpha_bar(true)
                            .inputs(false)
                            .label(false)
                            .build();
                        ui.same_line();
                        ui.text("Team Color");

                        ui.color_edit4_config("Enemy Color", &mut settings.esp_color_enemy)
                            .alpha_bar(true)
                            .inputs(false)
                            .label(false)
                            .build();
                        ui.same_line();
                        ui.text("Team Color");
                    }

                    if let Some(_) = ui.tab_item("Aim Assist") {
                        ui.button_key_optional("Trigger Bot", &mut settings.key_trigger_bot, [150.0, 0.0]);
                        ui.checkbox("Team Check", &mut settings.trigger_bot_team_check);
                        ui.checkbox("Simle Recoil Helper", &mut settings.aim_assist_recoil);
                    }
                }
            });
    }
}