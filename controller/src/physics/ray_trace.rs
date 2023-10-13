use super::rubikon::Mesh;

pub struct Ray {
    pub origin: nalgebra::Vector3<f32>,
    pub direction: nalgebra::Vector3<f32>,
    pub max_distance: f32,
}

pub struct RayHit {
    pub location: nalgebra::Vector3<f32>,
    pub hit_triangle: usize,
    pub t: f32,
}

impl Ray {
    // https://stackoverflow.com/a/42752998/7588455
    fn intersection_mesh(&self, mesh: &Mesh, triangle_index: usize) -> Option<f32> {
        let triangle = mesh.triangles[triangle_index];
        let a = mesh.vertices[triangle.x as usize];
        let b = mesh.vertices[triangle.y as usize];
        let c = mesh.vertices[triangle.z as usize];

        let e1 = b - a;
        let e2 = c - a;
        let n = e1.cross(&e2);
        let det = -self.direction.dot(&n);
        let invdet = 1.0 / det;
        let ao = self.origin - a;
        let dao = ao.cross(&self.direction);

        let u = e2.dot(&dao) * invdet;
        let v = -e1.dot(&dao) * invdet;
        let t = ao.dot(&n) * invdet;

        if det >= 1e-6 && t >= 0.0 && u >= 0.0 && v >= 0.0 && (u + v) <= 1.0 {
            Some(t)
        } else {
            None
        }
    }

    pub fn trace(&self, mesh: &Mesh) -> Option<RayHit> {
        let mut result = RayHit {
            hit_triangle: 0,
            location: Default::default(),
            t: self.max_distance,
        };

        // FIXME: Use kd-tree
        for triangle_index in 0..mesh.triangles.len() {
            if let Some(v) = self.intersection_mesh(mesh, triangle_index) {
                if v < result.t {
                    result.t = v;
                    result.hit_triangle = triangle_index;
                }
            }
        }

        if result.t == self.max_distance {
            /* Nothing has been hit */
            return None;
        }

        result.location = self.origin + self.direction * result.t;
        Some(result)
    }
}
