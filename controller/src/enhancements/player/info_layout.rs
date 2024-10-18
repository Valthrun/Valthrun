use imgui::ImColor32;

pub struct PlayerInfoLayout<'a> {
    ui: &'a imgui::Ui,
    draw: &'a imgui::DrawListMut<'a>,

    vmin: nalgebra::Vector2<f32>,
    vmax: nalgebra::Vector2<f32>,

    line_count: usize,
    font_scale: f32,

    has_2d_box: bool,
}

impl<'a> PlayerInfoLayout<'a> {
    pub fn new(
        ui: &'a imgui::Ui,
        draw: &'a imgui::DrawListMut<'a>,
        screen_bounds: mint::Vector2<f32>,
        vmin: nalgebra::Vector2<f32>,
        vmax: nalgebra::Vector2<f32>,
        has_2d_box: bool,
    ) -> Self {
        let target_scale_raw = (vmax.y - vmin.y) / screen_bounds.y * 8.0;
        let target_scale = target_scale_raw.clamp(0.5, 1.25);
        ui.set_window_font_scale(target_scale);

        Self {
            ui,
            draw,

            vmin,
            vmax,

            line_count: 0,
            font_scale: target_scale,

            has_2d_box,
        }
    }

    pub fn add_line(&mut self, color: impl Into<ImColor32>, text: &str) {
        let [text_width, _] = self.ui.calc_text_size(text);

        let mut pos = if self.has_2d_box {
            let mut pos = self.vmin;
            pos.x = self.vmax.x + 5.0;
            pos
        } else {
            let mut pos = self.vmax.clone();
            pos.x -= (self.vmax.x - self.vmin.x) / 2.0;
            pos.x -= text_width / 2.0;
            pos
        };
        pos.y += self.line_count as f32 * self.font_scale * (self.ui.text_line_height())
            + 4.0 * self.line_count as f32;

        self.draw.add_text([pos.x, pos.y], color, text);
        self.line_count += 1;
    }
}

impl Drop for PlayerInfoLayout<'_> {
    fn drop(&mut self) {
        self.ui.set_window_font_scale(1.0);
    }
}
