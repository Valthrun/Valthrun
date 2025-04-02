use std::borrow::Cow;

use imgui::{
    DrawListMut,
    ImColor32,
};

use crate::{
    settings::HotKey,
    UnicodeTextRenderer,
};

const TEXT_SHADOW_OFFSET: f32 = 1.0;
const TEXT_SHADOW_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 0.7];

pub trait TextWithShadowDrawList {
    fn add_text_with_shadow(&self, pos: [f32; 2], color: impl Into<ImColor32>, text: &str);
}

impl TextWithShadowDrawList for DrawListMut<'_> {
    fn add_text_with_shadow(&self, pos: [f32; 2], color: impl Into<ImColor32>, text: &str) {
        self.add_text(
            [pos[0] + TEXT_SHADOW_OFFSET, pos[1] + TEXT_SHADOW_OFFSET],
            TEXT_SHADOW_COLOR,
            text,
        );

        self.add_text([pos[0], pos[1]], color, text);
    }
}

pub trait TextWithShadowUi {
    fn text_with_shadow(&self, text: &str);
    fn text_colored_with_shadow(&self, color: impl Into<ImColor32>, text: &str);
}

impl TextWithShadowUi for imgui::Ui {
    fn text_with_shadow(&self, text: &str) {
        let pos = self.cursor_pos();

        self.set_cursor_pos([pos[0] + TEXT_SHADOW_OFFSET, pos[1] + TEXT_SHADOW_OFFSET]);
        self.text_colored(TEXT_SHADOW_COLOR, text);

        self.set_cursor_pos(pos);
        self.text(text);
    }

    fn text_colored_with_shadow(&self, color: impl Into<ImColor32>, text: &str) {
        let pos = self.cursor_pos();
        let color = color.into();
        let color_vec = [
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0,
        ];

        self.set_cursor_pos([pos[0] + TEXT_SHADOW_OFFSET, pos[1] + TEXT_SHADOW_OFFSET]);
        self.text_colored(TEXT_SHADOW_COLOR, text);

        self.set_cursor_pos(pos);
        self.text_colored(color_vec, text);
    }
}

pub trait UnicodeTextWithShadowUi {
    fn unicode_text_with_shadow(&self, unicode_text: &UnicodeTextRenderer, text: &str);

    fn unicode_text_colored_with_shadow(
        &self,
        unicode_text: &UnicodeTextRenderer,
        color: impl Into<ImColor32>,
        text: &str,
    );
}

impl UnicodeTextWithShadowUi for imgui::Ui {
    fn unicode_text_with_shadow(&self, unicode_text: &UnicodeTextRenderer, text: &str) {
        let pos = self.cursor_pos();

        self.set_cursor_pos([pos[0] + TEXT_SHADOW_OFFSET, pos[1] + TEXT_SHADOW_OFFSET]);
        unicode_text.text_colored(TEXT_SHADOW_COLOR, text);

        self.set_cursor_pos(pos);
        unicode_text.text(text);
    }

    fn unicode_text_colored_with_shadow(
        &self,
        unicode_text: &UnicodeTextRenderer,
        color: impl Into<ImColor32>,
        text: &str,
    ) {
        let pos = self.cursor_pos();
        let color = color.into();
        let color_vec = [
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0,
        ];

        self.set_cursor_pos([pos[0] + TEXT_SHADOW_OFFSET, pos[1] + TEXT_SHADOW_OFFSET]);
        unicode_text.text_colored(TEXT_SHADOW_COLOR, text);

        self.set_cursor_pos(pos);
        unicode_text.text_colored(color_vec, text);
    }
}

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
