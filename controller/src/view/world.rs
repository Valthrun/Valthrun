use cs2::{
    CS2HandleState,
    CS2Offsets,
};
use imgui::ImColor32;
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

/// View controller which helps resolve in game
/// coordinates into 2d screen coordinates.
pub struct ViewController {
    view_matrix: nalgebra::Matrix4<f32>,
    pub screen_bounds: mint::Vector2<f32>,
}

impl State for ViewController {
    type Parameter = ();

    fn create(_states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        Ok(Self {
            view_matrix: Default::default(),
            screen_bounds: mint::Vector2 { x: 0.0, y: 0.0 },
        })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Persistent
    }

    fn update(&mut self, states: &StateRegistry) -> anyhow::Result<()> {
        let cs2 = states.resolve::<CS2HandleState>(())?;
        let offsets = states.resolve::<CS2Offsets>(())?;

        self.view_matrix = cs2.read_sized(&[offsets.view_matrix])?;
        Ok(())
    }
}

impl ViewController {
    pub fn update_screen_bounds(&mut self, bounds: mint::Vector2<f32>) {
        self.screen_bounds = bounds;
    }

    pub fn get_camera_world_position(&self) -> Option<nalgebra::Vector3<f32>> {
        let view_matrix = self.view_matrix;
        let a = view_matrix.m22 * view_matrix.m33 - view_matrix.m32 * view_matrix.m23;
        let b = view_matrix.m32 * view_matrix.m13 - view_matrix.m12 * view_matrix.m33;
        let c = view_matrix.m12 * view_matrix.m23 - view_matrix.m22 * view_matrix.m13;
        let z = view_matrix.m11 * a + view_matrix.m21 * b + view_matrix.m31 * c;

        if z.abs() < 0.0001 {
            return None;
        }

        let d = view_matrix.m31 * view_matrix.m23 - view_matrix.m21 * view_matrix.m33;
        let e = view_matrix.m11 * view_matrix.m33 - view_matrix.m31 * view_matrix.m13;
        let f = view_matrix.m21 * view_matrix.m13 - view_matrix.m11 * view_matrix.m23;
        let g = view_matrix.m21 * view_matrix.m32 - view_matrix.m31 * view_matrix.m22;
        let h = view_matrix.m31 * view_matrix.m12 - view_matrix.m11 * view_matrix.m32;
        let k = view_matrix.m11 * view_matrix.m22 - view_matrix.m21 * view_matrix.m12;

        let x = (a * view_matrix.m41 + d * view_matrix.m42 + g * view_matrix.m43) / z;
        let y = (b * view_matrix.m41 + e * view_matrix.m42 + h * view_matrix.m43) / z;
        let z = (c * view_matrix.m41 + f * view_matrix.m42 + k * view_matrix.m43) / z;

        Some(nalgebra::Vector3::new(-x, -y, -z))
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
}
