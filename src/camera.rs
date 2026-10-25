use glam::{Mat4, Quat, Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum View {
    Front,
    Back,
    Left,
    Right,
    Top,
    Bottom,
}
impl View {
    pub const ALL: [Self; 6] = [
        Self::Front,
        Self::Back,
        Self::Left,
        Self::Right,
        Self::Top,
        Self::Bottom,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Front => "Front",
            Self::Back => "Back",
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Top => "Top",
            Self::Bottom => "Bottom",
        }
    }
    pub fn direction(self) -> Vec3 {
        match self {
            Self::Front => -Vec3::Y,
            Self::Back => Vec3::Y,
            Self::Left => -Vec3::X,
            Self::Right => Vec3::X,
            Self::Top => Vec3::Z,
            Self::Bottom => -Vec3::Z,
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct ViewportRect {
    pub left: u32,
    pub bottom: u32,
    pub width: u32,
    pub height: u32,
}
impl ViewportRect {
    pub fn below_menu(size: (u32, u32), menu_points: f32, pixels_per_point: f32) -> Self {
        Self {
            left: 0,
            bottom: 0,
            width: size.0.max(1),
            height: size
                .1
                .saturating_sub((menu_points * pixels_per_point).round() as u32)
                .max(1),
        }
    }

    pub fn aspect(self) -> f32 {
        self.width.max(1) as f32 / self.height.max(1) as f32
    }
}
pub struct Camera {
    pub rotation: Quat,
    pub pan: [f32; 2],
    pub zoom: f32,
}
impl Default for Camera {
    fn default() -> Self {
        let mut c = Self {
            rotation: Quat::IDENTITY,
            pan: [0.; 2],
            zoom: 1.,
        };
        c.rotation = Quat::from_mat4(&Mat4::look_at_rh(
            Vec3::new(1., -1., 1.),
            Vec3::ZERO,
            Vec3::Z,
        ))
        .normalize();
        c
    }
}
impl Camera {
    pub fn set_view(&mut self, view: View) {
        let up = match view {
            View::Top => Vec3::Y,
            View::Bottom => -Vec3::Y,
            _ => Vec3::Z,
        };
        self.rotation =
            Quat::from_mat4(&Mat4::look_at_rh(view.direction(), Vec3::ZERO, up)).normalize();
    }
    pub fn orbit(&mut self, dx: f32, dy: f32) {
        self.rotation =
            (Quat::from_rotation_x(dy * 0.008) * Quat::from_rotation_y(dx * 0.008) * self.rotation)
                .normalize();
    }
    pub fn vertical_span(&self) -> f64 {
        2.8 / self.zoom as f64
    }
    pub fn matrix(&self, viewport: ViewportRect) -> Mat4 {
        let h = self.vertical_span() as f32 * 0.5;
        Mat4::orthographic_rh_gl(
            -h * viewport.aspect(),
            h * viewport.aspect(),
            -h,
            h,
            0.01,
            100.,
        ) * Mat4::from_translation(Vec3::new(self.pan[0], self.pan[1], -5.))
            * Mat4::from_quat(self.rotation)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn standard_views() {
        let mut c = Camera::default();
        for v in View::ALL {
            c.set_view(v);
            assert!((c.rotation * v.direction() - Vec3::Z).length() < 1e-5);
            let up = if v == View::Top {
                Vec3::Y
            } else if v == View::Bottom {
                -Vec3::Y
            } else {
                Vec3::Z
            };
            assert!((c.rotation * up - Vec3::Y).length() < 1e-5);
        }
    }
    #[test]
    fn standard_view_preserves_framing() {
        let mut c = Camera {
            pan: [0.3, -0.2],
            zoom: 3.,
            ..Default::default()
        };
        for view in View::ALL {
            c.set_view(view);
            assert_eq!(c.pan, [0.3, -0.2]);
            assert_eq!(c.zoom, 3.);
        }
    }
    #[test]
    fn orbit_normalizes() {
        let mut c = Camera::default();
        for _ in 0..10000 {
            c.orbit(3., 4.);
        }
        assert!((c.rotation.length() - 1.).abs() < 1e-6);
    }
    #[test]
    fn viewport_excludes_menu_at_fractional_dpi() {
        let v = ViewportRect::below_menu((1500, 1140), 30., 1.5);
        assert_eq!(v.height, 1095);
        assert_eq!(ViewportRect::below_menu((1, 1), 30., 2.).height, 1);
    }
    #[test]
    fn viewport_aspect() {
        assert_eq!(
            ViewportRect {
                left: 0,
                bottom: 0,
                width: 800,
                height: 400
            }
            .aspect(),
            2.
        );
    }
}
