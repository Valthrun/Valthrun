use std::sync::Arc;

use cs2::{CS2Offsets, CS2Handle, Module};

/// View controller which helps resolve in game
/// coordinates into 2d screen coordinates.
pub struct ViewController {
    cs2_view_matrix_offset: u64,
    view_matrix: nalgebra::Matrix4<f32>,
    screen_bounds: mint::Vector2<f32>,
}

impl ViewController {
    pub fn new(offsets: Arc<CS2Offsets>) -> Self {
        Self {
            cs2_view_matrix_offset: offsets.view_matrix,
            view_matrix: Default::default(),
            screen_bounds: mint::Vector2 { x: 0.0, y: 0.0 },
        }
    }

    pub fn update_screen_bounds(&mut self, bounds: mint::Vector2<f32>) {
        self.screen_bounds = bounds;
    }

    pub fn update_view_matrix(&mut self, cs2: &CS2Handle) -> anyhow::Result<()> {
        self.view_matrix = cs2.read(Module::Client, &[self.cs2_view_matrix_offset])?;
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
}