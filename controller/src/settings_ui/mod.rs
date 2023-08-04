use std::{rc::Rc, cell::RefCell};

use imgui::{Condition, StyleVar};
use obfstr::obfstr;

use crate::{settings::AppSettings, Application};

pub struct SettingsUI {
    settings: Rc<RefCell<AppSettings>>,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
impl SettingsUI {
    pub fn new(settings: Rc<RefCell<AppSettings>>) -> Self {
        Self {
            settings
        }
    }

    pub fn render(&mut self, app: &Application, ui: &imgui::Ui) {
        ui.window(obfstr!("Valthrun"))
            .size([600.0, 300.0], Condition::FirstUseEver)
            .build(|| {
                if let Some(_tab_bar) = ui.tab_bar("main") {
                    if let Some(_tab) = ui.tab_item("Information") {
                        ui.text(obfstr!("Valthrun an open source CS2 external read only kernel cheat."));
                        ui.text(&format!("Valthrun Version {}", VERSION));
                        ui.text(&format!("CS2 Version {} ({})", app.cs2_build_info.revision, app.cs2_build_info.build_datetime));
                    }
                    
                    if let Some(_tab) = ui.tab_item("Visuals") {
                        let mut settings = self.settings.borrow_mut();
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
                }
            });
    }
}