use std::{
    cell::RefCell,
    collections::btree_map::Entry,
    rc::Rc,
    sync::atomic::Ordering,
    time::Instant,
};

use imgui::{
    Condition,
    ImColor32,
    SelectableFlags,
    StyleColor,
    StyleVar,
    TableColumnFlags,
    TableColumnSetup,
    TableFlags,
    TreeNodeFlags,
};
use obfstr::obfstr;

use super::{
    Color,
    EspColor,
    EspColorType,
    EspBombColor,
    EspBombColorType,
    EspConfig,
    EspSelector,
};
use crate::{
    settings::{
        AppSettings,
        EspBombSettings,
        EspBoxType,
        EspHealthBar,
        EspPlayerSettings,
        EspTracePosition,
    },
    utils::{
        ImGuiKey,
        ImguiComboEnum,
    },
    Application,
};

enum EspPlayerActiveHeader {
    Features,
    Style,
}

pub struct SettingsUI {
    settings: Rc<RefCell<AppSettings>>,
    discord_link_copied: Option<Instant>,

    esp_selected_target: EspSelector,
    esp_pending_target: Option<EspSelector>,

    esp_player_active_header: EspPlayerActiveHeader,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
impl SettingsUI {
    pub fn new(settings: Rc<RefCell<AppSettings>>) -> Self {
        Self {
            settings,
            discord_link_copied: None,

            esp_selected_target: EspSelector::None,
            esp_pending_target: None,

            esp_player_active_header: EspPlayerActiveHeader::Features,
        }
    }

    pub fn render(&mut self, app: &Application, ui: &imgui::Ui) {
        let content_font = ui.current_font().id();
        let _title_font = ui.push_font(app.fonts.valthrun);

        ui.window(obfstr!("Valthrun"))
            .size([600.0, 300.0], Condition::FirstUseEver)
            .title_bar(false)
            .build(|| {
                {
                    for (text, color) in [
                        ("V", [0.81, 0.69, 0.06, 1.0]),
                        ("a", [0.84, 0.61, 0.15, 1.0]),
                        ("l", [0.86, 0.52, 0.24, 1.0]),
                        ("t", [0.89, 0.44, 0.33, 1.0]),
                        ("h", [0.92, 0.36, 0.41, 1.0]),
                        ("r", [0.95, 0.27, 0.50, 1.0]),
                        ("u", [0.97, 0.19, 0.59, 1.0]),
                        ("n", [1.00, 0.11, 0.68, 1.0]),
                    ] {
                        ui.text_colored(color, text);
                        ui.same_line();
                    }

                    ui.new_line();
                    ui.dummy([ 0.0, 5.0 ]);
                }

                let _content_font = ui.push_font(content_font);
                let settings = self.settings.clone();
                let mut settings: std::cell::RefMut<'_, AppSettings> = settings.borrow_mut();
                if let Some(_tab_bar) = ui.tab_bar("main") {
                    if let Some(_tab) = ui.tab_item("Information") {
                        ui.text(obfstr!("Valthrun an open source CS2 external read only kernel gameplay enhancer."));
                        ui.text(&format!("{} Version {} ({})", obfstr!("Valthrun"), VERSION, env!("BUILD_TIME")));
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
                        ui.button_key(obfstr!("Toggle Settings"), &mut settings.key_settings, [150.0, 0.0]);
                        ui.button_key_optional(obfstr!("ESP toggle"), &mut settings.esp_toogle, [ 150.0, 0.0 ]);
                    }


                    if let Some(_tab) = ui.tab_item("Visuals") {
                        ui.checkbox(obfstr!("ESP"), &mut settings.esp);
                        ui.checkbox(obfstr!("Bomb Timer"), &mut settings.bomb_timer);
                        ui.checkbox(obfstr!("Spectators List"), &mut settings.spectators_list);

                    if let Some(_tab) = ui.tab_item("ESP") {
                        if !settings.esp {
                            let _style = ui.push_style_color(StyleColor::Text, [ 1.0, 0.76, 0.03, 1.0 ]);
                            ui.text("ESP has been disabled.");
                            ui.text("Please enable ESP under \"Visuals\" \"ESP\"");
                        } else {
                            self.render_esp_settings(&mut *settings, ui);
                        }
                    }

                    if let Some(_tab) = ui.tab_item("Bomb") {
                        if !settings.bomb_timer {
                            let _style = ui.push_style_color(StyleColor::Text, [ 1.0, 0.76, 0.03, 1.0 ]);
                            ui.text("Bomb has been disabled.");
                            ui.text("Please enable ESP under \"Visuals\" \"Bomb Timer\"");
                        } else {
                            self.render_bomb_esp_settings(&mut *settings, ui);
                        }
                    }

                    if let Some(_) = ui.tab_item(obfstr!("Aim Assist")) {
                        ui.checkbox(obfstr!("Trigger Bot Always Active"), &mut settings.trigger_bot_always_active);
                        ui.button_key_optional(obfstr!("Trigger Bot"), &mut settings.key_trigger_bot, [150.0, 0.0]);
                        if settings.trigger_bot_always_active || settings.key_trigger_bot.is_some() {
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
    
    fn get_esp_config_player<'a>(
        settings: &'a mut AppSettings,
        ui: &'a imgui::Ui,
        target: EspSelector,
    ) -> (&'a mut EspPlayerSettings, bool) {
        let config_key = target.config_key();
        let config_enabled = settings
            .esp_settings_enabled
            .get(&config_key)
            .cloned()
            .unwrap_or_default();
    
        let config = match settings.esp_settings.entry(config_key.clone()) {
            Entry::Occupied(entry) => {
                let value = entry.into_mut();
                if let EspConfig::Player(value) = value {
                    value
                } else {
                    log::warn!("Detected invalid player config for {}", config_key);
                    *value = EspConfig::Player(EspPlayerSettings::new(&target));
                    if let EspConfig::Player(value) = value {
                        value
                    } else {
                        unreachable!()
                    }
                }
            }
            Entry::Vacant(entry) => {
                if let EspConfig::Player(value) = entry.insert(EspConfig::Player(EspPlayerSettings::new(&target))) {
                    value
                } else {
                    unreachable!()
                }
            }
        };
        let _ui_enable_token = ui.begin_enabled(config_enabled);
    
        (config, config_enabled)
    }
    
    
    fn render_esp_target(
        &mut self,
        settings: &mut AppSettings,
        ui: &imgui::Ui,
        target: &EspSelector,
    ) {
        let config_key = target.config_key();
        let target_enabled = settings
            .esp_settings_enabled
            .get(&config_key)
            .cloned()
            .unwrap_or_default();

        let parent_enabled = target_enabled || {
            let mut current = target.parent();
            while let Some(parent) = current.take() {
                let enabled = settings
                    .esp_settings_enabled
                    .get(&parent.config_key())
                    .cloned()
                    .unwrap_or_default();

                if enabled {
                    current = Some(parent);
                    break;
                }

                current = parent.parent();
            }

            current.is_some()
        };

        {
            let pos_begin = ui.cursor_screen_pos();
            let clicked = ui
                .selectable_config(format!(
                    "{} ##{}",
                    target.config_display(),
                    target.config_key()
                ))
                .selected(target == &self.esp_selected_target)
                .flags(SelectableFlags::SPAN_ALL_COLUMNS)
                .build();

            let indicator_color = if target_enabled {
                ImColor32::from_rgb(0x4C, 0xAF, 0x50)
            } else if parent_enabled {
                ImColor32::from_rgb(0xFF, 0xC1, 0x07)
            } else {
                ImColor32::from_rgb(0xF4, 0x43, 0x36)
            };
            let pos_end = ui.cursor_screen_pos();
            let indicator_radius = ui.current_font_size() * 0.25;

            ui.get_window_draw_list()
                .add_circle(
                    [
                        pos_begin[0] - indicator_radius - 5.0,
                        pos_begin[1] + (pos_end[1] - pos_begin[1]) / 2.0 - indicator_radius / 2.0,
                    ],
                    indicator_radius,
                    indicator_color,
                )
                .filled(true)
                .build();

            if clicked {
                self.esp_pending_target = Some(target.clone());
            }
        }

        let children = target.children();
        if children.len() > 0 {
            ui.indent();
            for child in children.iter() {
                self.render_esp_target(settings, ui, child);
            }
            ui.unindent();
        }
    }

    fn render_esp_settings_player(
        &mut self,
        mut settings: &mut AppSettings,
        ui: &imgui::Ui,
        target: EspSelector,
    ) {
        let (config, config_enabled) = Self::get_esp_config_player(&mut settings, &ui, target);
        let _ui_enable_token = ui.begin_enabled(config_enabled);


        let content_height =
            ui.content_region_avail()[1] - ui.text_line_height_with_spacing() * 2.0 - 16.0;
        unsafe {
            imgui::sys::igSetNextItemOpen(
                matches!(
                    self.esp_player_active_header,
                    EspPlayerActiveHeader::Features
                ),
                0,
            );
        };
        if ui.collapsing_header("Features", TreeNodeFlags::empty()) {
            self.esp_player_active_header = EspPlayerActiveHeader::Features;
            if let Some(_token) = {
                ui.child_window("features")
                    .size([0.0, content_height])
                    .begin()
            } {
                ui.indent_by(5.0);
                ui.dummy([0.0, 5.0]);

                const COMBO_WIDTH: f32 = 150.0;
                {
                    const ESP_BOX_TYPES: [(EspBoxType, &'static str); 3] = [
                        (EspBoxType::None, "No"),
                        (EspBoxType::Box2D, "2D"),
                        (EspBoxType::Box3D, "3D"),
                    ];

                    ui.set_next_item_width(COMBO_WIDTH);
                    ui.combo_enum(obfstr!("player box"), &ESP_BOX_TYPES, &mut config.box_type);
                }

                {
                    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
                    enum PlayerSkeletonType {
                        None,
                        Skeleton,
                    }

                    const PLAYER_SKELETON_TYPES: [(PlayerSkeletonType, &'static str); 2] = [
                        (PlayerSkeletonType::None, "No"),
                        (PlayerSkeletonType::Skeleton, "Show"),
                    ];

                    let mut skeleton_type = if config.skeleton {
                        PlayerSkeletonType::Skeleton
                    } else {
                        PlayerSkeletonType::None
                    };

                    ui.set_next_item_width(COMBO_WIDTH);
                    let value_changed = ui.combo_enum(
                        obfstr!("player skeleton"),
                        &PLAYER_SKELETON_TYPES,
                        &mut skeleton_type,
                    );

                    if value_changed {
                        config.skeleton = matches!(skeleton_type, PlayerSkeletonType::Skeleton);
                    }
                }

                {
                    const TRACER_LINE_TYPES: [(EspTracePosition, &'static str); 7] = [
                        (EspTracePosition::None, "No"),
                        (EspTracePosition::TopLeft, "Top left"),
                        (EspTracePosition::TopCenter, "Top (center)"),
                        (EspTracePosition::TopRight, "Top right"),
                        (EspTracePosition::BottomLeft, "Bottom left"),
                        (EspTracePosition::BottomCenter, "Bottom (center)"),
                        (EspTracePosition::BottomRight, "Bottom right"),
                    ];

                    ui.set_next_item_width(COMBO_WIDTH);
                    ui.combo_enum(
                        obfstr!("tracer lines"),
                        &TRACER_LINE_TYPES,
                        &mut config.tracer_lines,
                    );
                }

                {
                    const HEALTH_BAR_TYPES: [(EspHealthBar, &'static str); 5] = [
                        (EspHealthBar::None, "No"),
                        (EspHealthBar::Top, "Top"),
                        (EspHealthBar::Left, "Left"),
                        (EspHealthBar::Bottom, "Bottom"),
                        (EspHealthBar::Right, "Right"),
                    ];

                    ui.set_next_item_width(COMBO_WIDTH);
                    ui.combo_enum(
                        obfstr!("player health bar"),
                        &HEALTH_BAR_TYPES,
                        &mut config.health_bar,
                    );
                }
                ui.dummy([0.0, 10.0]);

                ui.text("Player Info");
                ui.checkbox(obfstr!("Name"), &mut config.info_name);
                ui.checkbox(obfstr!("Weapon"), &mut config.info_weapon);
                ui.checkbox(obfstr!("Distance"), &mut config.info_distance);
                ui.checkbox(obfstr!("Health"), &mut config.info_hp_text);
                ui.checkbox(obfstr!("Kit"), &mut config.info_kit);
            }
        }

        unsafe {
            imgui::sys::igSetNextItemOpen(
                matches!(self.esp_player_active_header, EspPlayerActiveHeader::Style),
                0,
            );
        };
        if ui.collapsing_header("Style & Colors", TreeNodeFlags::empty()) {
            self.esp_player_active_header = EspPlayerActiveHeader::Style;
            if let Some(_token) = {
                ui.child_window("styles")
                    .size([0.0, content_height])
                    .begin()
            } {
                ui.indent_by(5.0);
                ui.dummy([0.0, 5.0]);

                if let Some(_token) = {
                    let mut column_type = TableColumnSetup::new("Type");
                    column_type.init_width_or_weight = 100.0;
                    column_type.flags = TableColumnFlags::WIDTH_FIXED;

                    let mut column_value = TableColumnSetup::new("Value");
                    column_value.init_width_or_weight = 100.0;
                    column_value.flags = TableColumnFlags::WIDTH_FIXED;

                    ui.begin_table_header_with_flags(
                        "styles_table",
                        [TableColumnSetup::new("Name"), column_type, column_value],
                        TableFlags::ROW_BG
                            | TableFlags::BORDERS
                            | TableFlags::SIZING_STRETCH_PROP
                            | TableFlags::SCROLL_Y,
                    )
                } {
                    ui.table_next_row();
                    Self::render_esp_settings_player_style_color(
                        ui,
                        obfstr!("ESP box color"),
                        &mut config.box_color,
                    );

                    ui.table_next_row();
                    Self::render_esp_settings_player_style_width(
                        ui,
                        obfstr!("ESP box width"),
                        1.0,
                        10.0,
                        &mut config.box_width,
                    );

                    ui.table_next_row();
                    Self::render_esp_settings_player_style_color(
                        ui,
                        obfstr!("Player skeleton color"),
                        &mut config.skeleton_color,
                    );

                    ui.table_next_row();
                    Self::render_esp_settings_player_style_width(
                        ui,
                        obfstr!("Player skeleton width"),
                        1.0,
                        10.0,
                        &mut config.skeleton_width,
                    );

                    ui.table_next_row();
                    Self::render_esp_settings_player_style_width(
                        ui,
                        obfstr!("Health bar width"),
                        5.0,
                        30.0,
                        &mut config.health_bar_width,
                    );

                    ui.table_next_row();
                    Self::render_esp_settings_player_style_color(
                        ui,
                        obfstr!("Tracer line color"),
                        &mut config.tracer_lines_color,
                    );

                    ui.table_next_row();
                    Self::render_esp_settings_player_style_width(
                        ui,
                        obfstr!("Tracer line width"),
                        1.0,
                        10.0,
                        &mut config.tracer_lines_width,
                    );

                    ui.table_next_row();
                    Self::render_esp_settings_player_style_color(
                        ui,
                        obfstr!("Color info name"),
                        &mut config.info_name_color,
                    );

                    ui.table_next_row();
                    Self::render_esp_settings_player_style_color(
                        ui,
                        obfstr!("Color info distance"),
                        &mut config.info_distance_color,
                    );

                    ui.table_next_row();
                    Self::render_esp_settings_player_style_color(
                        ui,
                        obfstr!("Color info weapon"),
                        &mut config.info_weapon_color,
                    );

                    ui.table_next_row();
                    Self::render_esp_settings_player_style_color(
                        ui,
                        obfstr!("Color info health"),
                        &mut config.info_hp_text_color,
                    );

                    ui.table_next_row();
                    Self::render_esp_settings_player_style_color(
                        ui,
                        obfstr!("Color info kit"),
                        &mut config.info_kit_color,
                    );
                }
            }
        }

        drop(_ui_enable_token);
    }

    fn render_esp_settings_player_style_width(
        ui: &imgui::Ui,
        label: &str,
        min: f32,
        max: f32,
        value: &mut f32,
    ) -> bool {
        ui.table_next_column();
        ui.text(label);

        ui.table_next_column();
        ui.text(&format!("{:.2} - {:.2}", min, max));

        ui.table_next_column();
        if {
            ui.input_float(&format!("##{}_style_width", ui.table_row_index()), value)
                .build()
        } {
            *value = value.clamp(min, max);
            true
        } else {
            false
        }
    }

    fn render_esp_settings_player_style_color(ui: &imgui::Ui, label: &str, color: &mut EspColor) {
        ui.table_next_column();
        ui.text(label);

        ui.table_next_column();
        {
            let mut color_type = EspColorType::from_esp_color(color);
            ui.set_next_item_width(ui.content_region_avail()[0]);
            let color_type_changed = ui.combo_enum(
                &format!("##{}_color_type", ui.table_row_index()),
                &[
                    (EspColorType::Static, "Static"),
                    (EspColorType::HealthBased, "Health based"),
                    (EspColorType::HealthBasedRainbow, "Rainbow"),
                ],
                &mut color_type,
            );

            if color_type_changed {
                *color = match color_type {
                    EspColorType::Static => EspColor::Static {
                        value: Color::from_f32([1.0, 1.0, 1.0, 1.0]),
                    },
                    EspColorType::HealthBased => EspColor::HealthBased {
                        max: Color::from_f32([0.0, 1.0, 0.0, 1.0]),
                        min: Color::from_f32([1.0, 0.0, 0.0, 1.0]),
                    },
                    EspColorType::HealthBasedRainbow => EspColor::HealthBasedRainbow,
                }
            }
        }

        ui.table_next_column();
        {
            match color {
                EspColor::HealthBasedRainbow => ui.text("Rainbow"),
                EspColor::Static { value } => {
                    let mut color_value = value.as_f32();

                    if {
                        ui.color_edit4_config(
                            &format!("##{}_static_value", ui.table_row_index()),
                            &mut color_value,
                        )
                        .alpha_bar(true)
                        .inputs(false)
                        .label(false)
                        .build()
                    } {
                        *value = Color::from_f32(color_value);
                    }
                }
                EspColor::HealthBased { max, min } => {
                    let mut max_value = max.as_f32();
                    if {
                        ui.color_edit4_config(
                            &format!("##{}_health_max", ui.table_row_index()),
                            &mut max_value,
                        )
                        .alpha_bar(true)
                        .inputs(false)
                        .label(false)
                        .build()
                    } {
                        *max = Color::from_f32(max_value);
                    }

                    ui.same_line();
                    ui.text(" => ");
                    ui.same_line();

                    let mut min_value = min.as_f32();
                    if {
                        ui.color_edit4_config(
                            &format!("##{}_health_min", ui.table_row_index()),
                            &mut min_value,
                        )
                        .alpha_bar(true)
                        .inputs(false)
                        .label(false)
                        .build()
                    } {
                        *min = Color::from_f32(min_value);
                    }
                }
            }
        }
    }

    fn render_esp_settings_bomb_style_color(ui: &imgui::Ui, label: &str, color: &mut EspBombColor) {
        ui.table_next_column();
        ui.text(label);
    
        ui.table_next_column();
        {
            let mut color_type = EspBombColorType::from_bomb_esp_color(color);
            ui.set_next_item_width(150.0);
            let color_type_changed = ui.combo_enum(
                &format!("##{}_color_type", ui.table_row_index()),
                &[
                    (EspBombColorType::Static, "Static"),
                    (EspBombColorType::Distance, "Distance"),
                    (EspBombColorType::TimeDetonation, "Time Detonation"),
                ],
                &mut color_type,
            );
    
            if color_type_changed {
                *color = match color_type {
                    EspBombColorType::Static => EspBombColor::Static {
                        value: Color::from_f32([1.0, 1.0, 1.0, 1.0]),
                    },
                    EspBombColorType::Distance => EspBombColor::Distance,
                    EspBombColorType::TimeDetonation => EspBombColor::TimeDetonation,
                }
            }
        }
    
        ui.table_next_column();
        ui.same_line();
    
        {
            match color {
                EspBombColor::Distance => ui.text("Distance"),
                EspBombColor::Static { value } => {
                    let mut color_value = value.as_f32();
    
                    if {
                        ui.color_edit4_config(
                            &format!("##{}_static_value", ui.table_row_index()),
                            &mut color_value,
                        )
                        .alpha_bar(true)
                        .inputs(false)
                        .label(false)
                        .build()
                    } {
                        *value = Color::from_f32(color_value);
                    }
                }
                EspBombColor::TimeDetonation => ui.text("Time Detonation"),
            }
        }
    }
    
    
    
    fn render_esp_settings_chicken(
        &mut self,
        _settings: &mut AppSettings,
        ui: &imgui::Ui,
        _target: EspSelector,
    ) {
        ui.text("Chicken!");
    }

    fn render_esp_settings_weapon(
        &mut self,
        _settings: &mut AppSettings,
        ui: &imgui::Ui,
        _target: EspSelector,
    ) {
        ui.text("Weapon!");
    }
    
    fn get_or_insert_bomb_config<'a>(
        settings: &'a mut AppSettings,
        ui: &'a imgui::Ui,
    ) -> (&'a mut EspBombSettings, bool) {
        let config_key = "bomb".to_string();
        let config_enabled = settings
            .esp_settings_enabled
            .get(&config_key)
            .cloned()
            .unwrap_or_default();
    
        let config = match settings.esp_settings.entry(config_key.clone()) {
            Entry::Occupied(entry) => {
                let value = entry.into_mut();
                if let EspConfig::Bomb(value) = value {
                    value
                } else {
                    log::warn!("Detected invalid bomb config for {}", config_key);
                    *value = EspConfig::Bomb(EspBombSettings::new());
                    if let EspConfig::Bomb(value) = value {
                        value
                    } else {
                        unreachable!()
                    }
                }
            }
            Entry::Vacant(entry) => {
                if let EspConfig::Bomb(value) = entry.insert(EspConfig::Bomb(EspBombSettings::new())) {
                    value
                } else {
                    unreachable!()
                }
            }
        };
        let _ui_enable_token = ui.begin_enabled(config_enabled);
    
        (config, config_enabled)
    }

    fn render_esp_settings_bomb(&mut self, mut settings: &mut AppSettings, ui: &imgui::Ui) {
        let (mut config, mut config_enabled) = Self::get_or_insert_bomb_config(&mut settings, &ui);
        let _ui_enable_token = ui.begin_enabled(config_enabled);
    
        let content_height = ui.content_region_avail()[1] - ui.text_line_height_with_spacing() * 2.0 - 16.0;
        if ui.collapsing_header("Features", TreeNodeFlags::empty()) {
            self.esp_player_active_header = EspPlayerActiveHeader::Features;
            if let Some(_token) = {
                ui.child_window("features")
                    .size([0.0, content_height])
                    .begin()
            } {
                ui.dummy([0.0, 10.0]);
    
                ui.text("Bomb Info");
                ui.checkbox(obfstr!("Planted C4 ESP"), &mut config.bomb_position);
                ui.checkbox(obfstr!("Bomb site"), &mut config.bomb_site);
                ui.checkbox(obfstr!("Bomb status"), &mut config.bomb_status);
            }
        

            if config.bomb_position {
                Self::render_esp_settings_bomb_style_color(
                    ui,
                    obfstr!("Bomb position color"),
                    &mut config.bomb_position_color,
                );
            }
        }
    
        drop(_ui_enable_token);
    }
    
    

    fn render_esp_settings(&mut self, settings: &mut AppSettings, ui: &imgui::Ui) {
        if let Some(target) = self.esp_pending_target.take() {
            self.esp_selected_target = target;
        }

        /* the left tree */
        let content_region = ui.content_region_avail();
        let original_style = ui.clone_style();
        let tree_width = (content_region[0] * 0.25).max(150.0);
        let content_width = (content_region[0] - tree_width - 5.0).max(300.0);

        {
            let target_key = self.esp_selected_target.config_key();
            let target_enabled = settings
                .esp_settings_enabled
                .entry(target_key.to_string())
                .or_insert(false);

            ui.text("ESP Target");
            ui.same_line_with_pos(
                original_style.window_padding[0] * 2.0
                    + tree_width
                    + original_style.window_border_size,
            );
            ui.checkbox(self.esp_selected_target.config_title(), target_enabled);

            let reset_text = "Reset config";
            let reset_text_width = ui.calc_text_size(&reset_text)[0];

            let total_width = ui.content_region_avail()[0] + 2.0;
            ui.same_line_with_pos(total_width - reset_text_width);

            let _enabled = ui.begin_enabled(*target_enabled);
            if ui.button(reset_text) {
                /* just removing the key will work as a default config will be emplaced later */
                settings.esp_settings.remove(&target_key);
            }
        };

        //ui.dummy([0.0, 10.0]);

        if let (Some(_token), _padding) = {
            let padding = ui.push_style_var(StyleVar::WindowPadding([
                0.0,
                original_style.window_padding[1],
            ]));
            let window = ui
                .child_window("ESP Target")
                .size([tree_width, 0.0])
                .border(true)
                .draw_background(true)
                .scroll_bar(true)
                .begin();

            (window, padding)
        } {
            ui.indent_by(
                original_style.window_padding[0] +
                /* for the indicator */
                ui.current_font_size() * 0.5 + 4.0,
            );

            self.render_esp_target(settings, ui, &EspSelector::Player);
            // self.render_esp_target(settings, ui, &EspSelector::Chicken);
            // self.render_esp_target(settings, ui, &EspSelector::Weapon)
        }
        ui.same_line();
        if let Some(_token) = {
            ui.child_window("Content")
                .size([content_width, 0.0])
                .scroll_bar(true)
                .begin()
        } {
            match &self.esp_selected_target {
                EspSelector::None => { /* render info box maybe */ }
                EspSelector::Player
                | EspSelector::PlayerTeam { .. }
                | EspSelector::PlayerTeamVisibility { .. } => {
                    self.render_esp_settings_player(settings, ui, self.esp_selected_target.clone())
                }
                EspSelector::Chicken => {
                    self.render_esp_settings_chicken(settings, ui, self.esp_selected_target.clone())
                }
                EspSelector::Weapon
                | EspSelector::WeaponGroup { .. }
                | EspSelector::WeaponSingle { .. } => {
                    self.render_esp_settings_weapon(settings, ui, self.esp_selected_target.clone())
                }
            }
        }
    }

    fn render_bomb_esp_settings(&mut self, settings: &mut AppSettings, ui: &imgui::Ui) {

    
        let content_region = ui.content_region_avail();
        let content_width = content_region[0];
    
        self.render_esp_settings_bomb(settings, ui);
    }
    
}
