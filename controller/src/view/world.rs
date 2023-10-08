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

    pub fn draw_health_bar_hori(
        &self,
        draw: &imgui::DrawListMut,
        player_health: i32,
        max_health: f32,
        bar_pos: [f32; 2],
        bar_height: f32,
        esp_color: [f32; 4],
        border_thickness: f32,
        border_color: [f32; 4],
    ) {
        let health_percentage = player_health as f32 / max_health as f32;
        let bar_width = health_percentage * 75.0;
        let bar_filled_color = esp_color;
        
        let bar_x = bar_pos[0];
        let bar_y = bar_pos[1];
        
        // fixed border
        let border_x1 = bar_x - border_thickness / 2.0;
        let border_x2 = bar_x + 75.0 + border_thickness / 2.0;
        let border_y1 = bar_y - border_thickness / 2.0;
        let border_y2 = bar_y + bar_height + border_thickness / 2.0;
        
        draw.add_line([border_x1, border_y1], [border_x2, border_y1], border_color)
            .thickness(border_thickness)
            .build();
        draw.add_line([border_x1, border_y1], [border_x1, border_y2], border_color)
            .thickness(border_thickness)
            .build();
        draw.add_line([border_x2, border_y1], [border_x2, border_y2], border_color)
            .thickness(border_thickness)
            .build();
        draw.add_line([border_x1, border_y2], [border_x2, border_y2], border_color)
            .thickness(border_thickness)
            .build();
        
        // draw health bar
        for i in 0..(bar_width as i32) {
            let x1 = bar_x + i as f32;
            let x2 = x1 + 1.0;
            let y1 = bar_y;
            let y2 = bar_y + bar_height;
            draw.add_line([x1, y1], [x2, y2], bar_filled_color)
                .thickness(bar_height)
                .build();
        }                                                
    }

    pub fn draw_health_bar_vert(
        &self,
        draw: &imgui::DrawListMut,
        player_health: i32,
        max_health: f32,
        bar_pos: [f32; 2],
        bar_height: f32,
        bar_width: f32,
        esp_color: [f32; 4],
        border_thickness: f32,
        border_color: [f32; 4],
    ) {
        let health_percentage = player_health as f32 / max_health as f32;
        let bar_height = health_percentage * 75.0;
        let bar_filled_color = esp_color;
        
        let bar_x = bar_pos[0];
        let bar_y = bar_pos[1];
        
        let border_x1 = bar_x - border_thickness / 2.0;
        let border_x2 = bar_x + bar_width + border_thickness / 2.0;
        let border_y1 = bar_y - 75.0 - border_thickness / 2.0;
        let border_y2 = bar_y + border_thickness / 2.0;
        
        draw.add_line([border_x1, border_y1], [border_x2, border_y1], border_color)
            .thickness(border_thickness)
            .build();
        
        draw.add_line([border_x1, border_y1], [border_x1, border_y2], border_color)
            .thickness(border_thickness)
            .build();
        
        draw.add_line([border_x2, border_y1], [border_x2, border_y2], border_color)
            .thickness(border_thickness)
            .build();
        
        for i in 0..(bar_height as i32) {
            let y1 = bar_y - i as f32;
            let y2 = y1 - 1.0;
            let x1 = bar_x;
            let x2 = bar_x + bar_width;
            draw.add_line([x1, y1], [x2, y2], bar_filled_color)
                .thickness(bar_width)
                .build();
        }                                                          
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
}
