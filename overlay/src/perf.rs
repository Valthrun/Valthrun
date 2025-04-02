use std::time::Instant;

use imgui::{
    ImColor32,
    StyleColor,
};

pub struct PerfTracker {
    history_length: usize,

    marker_total: Box<[f32]>,
    markers: Vec<Box<[f32]>>,
    marker_names: Vec<&'static str>,
    marker_index: usize,

    buffer_index: usize,
    finished_buffer_index: usize,

    marker_begin: Instant,
    marker_step_begin: Instant,

    initial_perf: bool,
}

impl PerfTracker {
    fn allocate_fixed_buffer(length: usize) -> Box<[f32]> {
        let mut buffer = Vec::new();
        buffer.resize(length, Default::default());
        buffer.into_boxed_slice()
    }

    pub fn new(history_length: usize) -> Self {
        Self {
            history_length,

            marker_total: Self::allocate_fixed_buffer(history_length),
            markers: Vec::new(),
            marker_names: Vec::new(),

            marker_index: 0,
            buffer_index: 0,
            finished_buffer_index: 0,

            marker_begin: Instant::now(),
            marker_step_begin: Instant::now(),

            initial_perf: true,
        }
    }

    pub fn begin(&mut self) {
        self.marker_begin = Instant::now();
        self.marker_step_begin = Instant::now();
        self.marker_index = 0;

        self.buffer_index += 1;
        self.buffer_index %= self.history_length;
    }

    pub fn mark(&mut self, label: &'static str) {
        if self.initial_perf {
            self.marker_names.push(label);
            self.markers
                .push(Self::allocate_fixed_buffer(self.history_length))
        } else {
            assert_eq!(self.marker_names.get(self.marker_index), Some(&label));
        }

        let elapsed = self.marker_step_begin.elapsed();
        self.marker_step_begin = Instant::now();

        self.markers[self.marker_index][self.buffer_index] = elapsed.as_micros() as f32 / 1000.0;
        self.marker_index += 1;
    }

    pub fn finish(&mut self, label: &'static str) {
        self.mark(label);
        self.marker_total[self.buffer_index] =
            self.marker_begin.elapsed().as_micros() as f32 / 1000.0;
        self.initial_perf = false;
        self.finished_buffer_index = self.buffer_index;
    }

    pub fn history_length(&self) -> usize {
        self.history_length
    }

    pub fn set_history_length(&mut self, length: usize) {
        if self.history_length == length {
            return;
        }

        self.history_length = length;
        self.marker_total = Self::allocate_fixed_buffer(self.history_length);
        for marker in self.markers.iter_mut() {
            *marker = Self::allocate_fixed_buffer(self.history_length);
        }

        self.buffer_index = 0;
        self.finished_buffer_index = 0;
    }
}

const BAR_COLORS: [ImColor32; 8] = [
    ImColor32::from_rgb(25, 179, 28),   // green
    ImColor32::from_rgb(179, 25, 25),   // redish
    ImColor32::from_rgb(128, 128, 128), // gray
    ImColor32::from_rgb(179, 112, 25),  // brown
    ImColor32::from_rgb(25, 179, 179),  // cyan
    ImColor32::from_rgb(179, 25, 127),  // purple
    ImColor32::from_rgb(255, 235, 59),  // yellow
    ImColor32::from_rgb(244, 67, 54),   // red
];

struct MarkerLabel {
    x: f32,
    y: f32,

    width: f32,
    height: f32,

    text: String,
}

impl PerfTracker {
    fn render_chart(&self, ui: &imgui::Ui, origin: [f32; 2], bounds: [f32; 2], value_max: f32) {
        let draw_list = ui.get_window_draw_list();

        draw_list
            .add_rect(
                origin,
                [origin[0] + bounds[0], origin[1] + bounds[1]],
                ui.style_color(StyleColor::FrameBg),
            )
            .filled(true)
            .build();

        let mut bar_heights = Vec::new();
        bar_heights.resize(self.history_length, 0.0);

        let bar_space = 0.0;
        let bar_width =
            (bounds[0] - (self.history_length - 1) as f32 * bar_space) / self.history_length as f32;

        for (marker_index, marker) in self.markers.iter().enumerate() {
            for bar_index in 0..self.history_length {
                let bar_height = marker[bar_index] * bounds[1] / value_max;
                let bar_y_offset = bar_heights[bar_index];
                let bar_x_offset = bar_index as f32 * (bar_width + bar_space);

                draw_list
                    .add_rect(
                        [
                            origin[0] + bar_x_offset,
                            origin[1] + bounds[1] - bar_y_offset - bar_height,
                        ],
                        [
                            origin[0] + bar_x_offset + bar_width,
                            origin[1] + bounds[1] - bar_y_offset,
                        ],
                        BAR_COLORS[marker_index % BAR_COLORS.len()],
                    )
                    .filled(true)
                    .build();

                bar_heights[bar_index] += bar_height;
            }
        }

        /* draw update line */
        {
            let x_offset = (self.finished_buffer_index + 1) as f32 * (bar_width + bar_space);
            draw_list
                .add_line(
                    [origin[0] + x_offset, origin[1]],
                    [origin[0] + x_offset, origin[1] + bounds[1]],
                    ImColor32::from_rgb(0xFF, 0x00, 0x00),
                )
                .thickness(2.0)
                .build();
        }
    }

    fn generate_marker_labels(&self, ui: &imgui::Ui, max_width: f32) -> Vec<MarkerLabel> {
        let mut result = Vec::with_capacity(self.history_length);

        /* generate labels */
        for (marker_index, marker_name) in self.marker_names.iter().enumerate() {
            let marker = &self.markers[marker_index];

            let avg = marker.iter().cloned().reduce(|a, b| a + b).unwrap_or(0.0)
                / self.history_length as f32;
            let var = marker
                .iter()
                .cloned()
                .map(|v| (v - avg) * (v - avg))
                .reduce(|a, b| a + b)
                .unwrap_or(0.0)
                / self.history_length as f32;

            let text = format!(
                "{} (max: {:.2}ms, avg: {:.2}ms, std: {:.2}ms)",
                marker_name,
                marker.iter().cloned().reduce(f32::max).unwrap_or(0.0),
                avg,
                var.sqrt()
            );

            let size = ui.calc_text_size(&text);
            result.push(MarkerLabel {
                x: 0.0,
                y: 0.0,

                width: size[0],
                height: size[1],

                text,
            });
        }

        /* layout labels */
        let mut current_x = 0.0;
        let mut current_y = 0.0;
        let label_spacing = 10.0;

        for label in result.iter_mut() {
            if current_x + label.width > max_width {
                current_x = 0.0;
                current_y += ui.text_line_height_with_spacing();
            }

            label.x = current_x;
            label.y = current_y;

            current_x += label.width + label_spacing;
        }

        result
    }

    pub fn render(&self, ui: &imgui::Ui, bounds: [f32; 2]) {
        let origin = ui.cursor_screen_pos();
        let marker_labels = self.generate_marker_labels(ui, bounds[0]);
        let label_height = marker_labels
            .iter()
            .map(|label| label.y + label.height)
            .reduce(f32::max)
            .unwrap_or(50.0);

        let chart_bounds = [bounds[0] - 75.0, bounds[1] - label_height - 5.0];
        let value_max = self
            .marker_total
            .iter()
            .cloned()
            .reduce(f32::max)
            .unwrap_or(1.0)
            .ceil();

        /* Render Y-axis labels */
        {
            let draw_list = ui.get_window_draw_list();
            for quantile in [1.0, 0.75, 0.5, 0.25] {
                let y_offset = origin[1] + chart_bounds[1] * (1.0 - quantile);
                let center_offset = if quantile >= 1.0 {
                    0.0
                } else {
                    draw_list
                        .add_line(
                            [origin[0], y_offset],
                            [origin[0] + chart_bounds[0], y_offset],
                            ImColor32::from_rgb(63, 63, 63),
                        )
                        .thickness(1.0)
                        .build();

                    ui.text_line_height() / 2.0
                };
                draw_list.add_text(
                    [origin[0] + chart_bounds[0] + 5.0, y_offset - center_offset],
                    ui.style_color(StyleColor::Text),
                    format!("{:.1} ms", value_max * quantile),
                );
            }
        }

        self.render_chart(ui, origin, chart_bounds, value_max);

        /* render bar labels */
        {
            let draw_list = ui.get_window_draw_list();
            for (label_index, label) in marker_labels.into_iter().enumerate() {
                draw_list.add_text(
                    [
                        origin[0] + label.x,
                        origin[1] + chart_bounds[1] + 5.0 + label.y,
                    ],
                    BAR_COLORS[label_index % BAR_COLORS.len()].to_rgba_f32s(),
                    label.text,
                );
            }
        }
    }
}
