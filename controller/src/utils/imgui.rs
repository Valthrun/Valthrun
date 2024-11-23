use std::borrow::Cow;

use crate::settings::HotKey;

pub trait ImguiUiEx {
    fn set_cursor_pos_x(&self, pos: f32);

    #[allow(unused)]
    fn set_cursor_pos_y(&self, pos: f32);
}

impl ImguiUiEx for imgui::Ui {
    fn set_cursor_pos_x(&self, pos: f32) {
        unsafe { imgui::sys::igSetCursorPosX(pos) };
    }

    fn set_cursor_pos_y(&self, pos: f32) {
        unsafe { imgui::sys::igSetCursorPosY(pos) };
    }
}

pub trait ImGuiKey {
    fn button_key(&self, label: &str, key: &mut HotKey, size: [f32; 2]) -> bool;
    fn button_key_optional(&self, label: &str, key: &mut Option<HotKey>, size: [f32; 2]) -> bool;
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

pub trait ImguiComboEnum {
    fn combo_enum<T: PartialEq + Copy>(
        &self,
        label: impl AsRef<str>,
        values: &[(T, &'static str)],
        value: &mut T,
    ) -> bool;
}

impl ImguiComboEnum for imgui::Ui {
    fn combo_enum<T: PartialEq + Copy>(
        &self,
        label: impl AsRef<str>,
        values: &[(T, &'static str)],
        value: &mut T,
    ) -> bool {
        let mut type_index = values
            .iter()
            .position(|(enum_value, _)| enum_value == value)
            .unwrap_or_default();

        fn display_name<'a, T>(entry: &'a (T, &'static str)) -> Cow<'a, str> {
            entry.1.into()
        }

        if self.combo(label, &mut type_index, values, &display_name) {
            *value = values[type_index].0;
            true
        } else {
            false
        }
    }
}

mod hotkey {
    use imgui::Key;

    use crate::settings::HotKey;

    pub fn render_button_key(
        ui: &imgui::Ui,
        label: &str,
        key: &mut Option<HotKey>,
        size: [f32; 2],
        optional: bool,
    ) -> bool {
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
            if ui.button_with_size(&button_label, [size[0] - 35.0, size[1]]) {
                ui.open_popup(label);
            }

            ui.same_line_with_spacing(0.0, 10.0);

            ui.disabled(key.is_none(), || {
                if ui.button_with_size("X", [25.0, 0.0]) {
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
