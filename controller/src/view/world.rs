use std::sync::Arc;

use cs2::{
    CS2Handle,
    CS2Offsets,
};
use imgui::ImColor32;

/// View controller which helps resolve in game
/// coordinates into 2d screen coordinates.
pub struct ViewController {
    cs2_view_matrix: u64,
    view_matrix: nalgebra::Matrix4<f32>,
    pub screen_bounds: mint::Vector2<f32>,
}

impl ViewController {
    pub fn new(offsets: Arc<CS2Offsets>) -> Self {
        Self {
            cs2_view_matrix: offsets.view_matrix,
            view_matrix: Default::default(),
            screen_bounds: mint::Vector2 { x: 0.0, y: 0.0 },
        }
    }

    pub fn update_screen_bounds(&mut self, bounds: mint::Vector2<f32>) {
        self.screen_bounds = bounds;
    }

    pub fn update_view_matrix(&mut self, cs2: &CS2Handle) -> anyhow::Result<()> {
        self.view_matrix = cs2.read_sized(&[self.cs2_view_matrix])?;
        Ok(())
    }

    /// Returning an mint::Vector2<f32> as the result should be used via ImGui.
    pub fn world_to_screen(
        &self,
        vec: &nalgebra::Vector3<f32>,
        allow_of_screen: bool,
    ) -> Option<mint::Vector2<f32>> {
        let screen_coords =
            nalgebra::Vector4::new(vec.x, vec.y, vec.z, 1.0).transpose() * self.view_matrix;

        if screen_coords.w < 0.1 {
            return None;
        }

        if !allow_of_screen
            && (screen_coords.x < -screen_coords.w
                || screen_coords.x > screen_coords.w
                || screen_coords.y < -screen_coords.w
                || screen_coords.y > screen_coords.w)
        {
            return None;
        }

        let mut screen_pos = mint::Vector2::from_slice(&[
            screen_coords.x / screen_coords.w,
            screen_coords.y / screen_coords.w,
        ]);
        screen_pos.x = (screen_pos.x + 1.0) * self.screen_bounds.x / 2.0;
        screen_pos.y = (-screen_pos.y + 1.0) * self.screen_bounds.y / 2.0;
        Some(screen_pos)
    }

    pub fn calculate_box_2d(
        &self,
        vmin: &nalgebra::Vector3<f32>,
        vmax: &nalgebra::Vector3<f32>,
    ) -> Option<(nalgebra::Vector2<f32>, nalgebra::Vector2<f32>)> {
        type Vec3 = nalgebra::Vector3<f32>;
        type Vec2 = nalgebra::Vector2<f32>;

        let points = [
            /* bottom */
            Vec3::new(vmin.x, vmin.y, vmin.z),
            Vec3::new(vmax.x, vmin.y, vmin.z),
            Vec3::new(vmin.x, vmax.y, vmin.z),
            Vec3::new(vmax.x, vmax.y, vmin.z),
            /* top */
            Vec3::new(vmin.x, vmin.y, vmax.z),
            Vec3::new(vmax.x, vmin.y, vmax.z),
            Vec3::new(vmin.x, vmax.y, vmax.z),
            Vec3::new(vmax.x, vmax.y, vmax.z),
        ];

        let mut min2d = Vec2::new(f32::MAX, f32::MAX);
        let mut max2d = Vec2::new(-f32::MAX, -f32::MAX);

        for point in points {
            if let Some(point) = self.world_to_screen(&point, true) {
                min2d.x = min2d.x.min(point.x);
                min2d.y = min2d.y.min(point.y);

                max2d.x = max2d.x.max(point.x);
                max2d.y = max2d.y.max(point.y);
            }
        }

        if min2d.x >= max2d.x {
            return None;
        }

        if min2d.y >= max2d.y {
            return None;
        }

        Some((min2d, max2d))
    }

    pub fn draw_box_3d(
        &self,
        draw: &imgui::DrawListMut,
        vmin: &nalgebra::Vector3<f32>,
        vmax: &nalgebra::Vector3<f32>,
        color: ImColor32,
        thickness: f32,
    ) {
        type Vec3 = nalgebra::Vector3<f32>;

        let lines = [
            /* bottom */
            (
                Vec3::new(vmin.x, vmin.y, vmin.z),
                Vec3::new(vmax.x, vmin.y, vmin.z),
            ),
            (
                Vec3::new(vmax.x, vmin.y, vmin.z),
                Vec3::new(vmax.x, vmin.y, vmax.z),
            ),
            (
                Vec3::new(vmax.x, vmin.y, vmax.z),
                Vec3::new(vmin.x, vmin.y, vmax.z),
            ),
            (
                Vec3::new(vmin.x, vmin.y, vmax.z),
                Vec3::new(vmin.x, vmin.y, vmin.z),
            ),
            /* top */
            (
                Vec3::new(vmin.x, vmax.y, vmin.z),
                Vec3::new(vmax.x, vmax.y, vmin.z),
            ),
            (
                Vec3::new(vmax.x, vmax.y, vmin.z),
                Vec3::new(vmax.x, vmax.y, vmax.z),
            ),
            (
                Vec3::new(vmax.x, vmax.y, vmax.z),
                Vec3::new(vmin.x, vmax.y, vmax.z),
            ),
            (
                Vec3::new(vmin.x, vmax.y, vmax.z),
                Vec3::new(vmin.x, vmax.y, vmin.z),
            ),
            /* corners */
            (
                Vec3::new(vmin.x, vmin.y, vmin.z),
                Vec3::new(vmin.x, vmax.y, vmin.z),
            ),
            (
                Vec3::new(vmax.x, vmin.y, vmin.z),
                Vec3::new(vmax.x, vmax.y, vmin.z),
            ),
            (
                Vec3::new(vmax.x, vmin.y, vmax.z),
                Vec3::new(vmax.x, vmax.y, vmax.z),
            ),
            (
                Vec3::new(vmin.x, vmin.y, vmax.z),
                Vec3::new(vmin.x, vmax.y, vmax.z),
            ),
        ];

        for (start, end) in lines {
            if let (Some(start), Some(end)) = (
                self.world_to_screen(&start, true),
                self.world_to_screen(&end, true),
            ) {
                draw.add_line(start, end, color)
                    .thickness(thickness)
                    .build();
            }
        }
    }

    pub fn draw_health_bar(
        &self,
        draw: &imgui::DrawListMut,
        bar_x: f32,
        bar_y: f32,
        filled_height: f32,
        bar_width: f32,
        health_color: [f32; 4],
        border_thickness: f32,
        border_color: [f32; 4],
        bar_height: f32,
        border_bar_y: f32,
    ) {
        //get fix edge
        let border_bottom_y = border_bar_y - border_thickness;
        let border_top_y = border_bottom_y - bar_height - border_thickness;

        //border
        draw.add_rect(
            [bar_x - border_thickness, border_top_y],
            [bar_x + bar_width + border_thickness, border_bottom_y],
            border_color,
        )
        .thickness(border_thickness)
        .build();

        draw.add_rect_filled_multicolor(
            [bar_x, bar_y],
            [bar_x + bar_width, bar_y + filled_height],
            health_color,
            health_color,
            health_color,
            health_color,
        );
    }

    pub fn calculate_rainbow_color(&self, value: f32) -> [f32; 4] {
        let frequency = 0.1;
        let r = (frequency * value).sin() * 127.0 + 128.0;
        let g = (frequency * value + 2.0 * std::f32::consts::PI / 3.0).sin() * 127.0 + 128.0;
        let b = (frequency * value + 4.0 * std::f32::consts::PI / 3.0).sin() * 127.0 + 128.0;
        [r / 255.0, g / 255.0, b / 255.0, 1.0]
    }

    pub fn calculate_health_color(&self, health_percentage: f32) -> [f32; 4] {
        if health_percentage > 0.6 {
            [
                2.0 - 2.0 * health_percentage,
                2.0 * health_percentage,
                0.0,
                1.0,
            ]
        } else if health_percentage > 0.3 {
            [1.0, 1.0, 2.0 - 2.0 * health_percentage, 1.0]
        } else {
            [1.0, 2.0 * health_percentage, 0.0, 1.0]
        }
    }
}
