use crate::camera::{Camera, View};
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use glam::{Quat, Vec3};

pub struct Face {
    pub view: View,
    pub points: Vec<Pos2>,
    pub depth: f32,
}
pub fn project(rotation: Quat, rect: Rect) -> Vec<Face> {
    let scale = rect.width().min(rect.height()) * 0.23;
    let mut faces = Vec::new();
    for view in View::ALL {
        let normal = view.direction();
        let rotated = rotation * normal;
        if rotated.z <= 1e-5 {
            continue;
        }
        let u = if normal.z.abs() > 0.5 {
            Vec3::X
        } else {
            normal.cross(Vec3::Z)
        };
        let v = normal.cross(u);
        let points = [(-1., -1.), (1., -1.), (1., 1.), (-1., 1.)]
            .map(|(a, b)| {
                let p = rotation * (normal + u * a + v * b);
                rect.center() + Vec2::new(p.x, -p.y) * scale
            })
            .to_vec();
        faces.push(Face {
            view,
            points,
            depth: rotated.z,
        });
    }
    faces.sort_by(|a, b| a.depth.total_cmp(&b.depth));
    faces
}
pub fn hit(faces: &[Face], p: Pos2) -> Option<View> {
    faces
        .iter()
        .rev()
        .find(|face| {
            let mut positive = false;
            let mut negative = false;
            for i in 0..4 {
                let a = face.points[i];
                let b = face.points[(i + 1) % 4];
                let c = (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
                positive |= c > 0.001;
                negative |= c < -0.001;
            }
            !(positive && negative)
        })
        .map(|f| f.view)
}
fn sphere(p: Pos2, rect: Rect) -> Vec3 {
    let v = (p - rect.center()) / (rect.width().min(rect.height()) * 0.5);
    let length = v.length_sq();
    if length <= 1. {
        Vec3::new(v.x, -v.y, (1. - length).sqrt())
    } else {
        Vec3::new(v.x, -v.y, 0.).normalize()
    }
}
#[derive(Default)]
pub struct ViewCube {
    previous: Option<Pos2>,
}
impl ViewCube {
    pub fn cancel(&mut self) {
        self.previous = None;
    }
    pub fn draw(&mut self, ui: &mut egui::Ui, camera: &mut Camera, enabled: bool) -> Option<View> {
        let (rect, response) = ui.allocate_exact_size(
            Vec2::splat(120.).min(ui.available_size()),
            egui::Sense::click_and_drag(),
        );
        let faces = project(camera.rotation, rect);
        let hover = response.hover_pos().and_then(|p| hit(&faces, p));
        for face in &faces {
            let color = if hover == Some(face.view) && enabled {
                Color32::from_rgb(100, 140, 170)
            } else {
                Color32::from_gray(65 + (face.depth * 35.) as u8)
            };
            ui.painter().add(egui::Shape::convex_polygon(
                face.points.clone(),
                color,
                Stroke::new(1., Color32::LIGHT_GRAY),
            ));
            let center = face
                .points
                .iter()
                .fold(Vec2::ZERO, |sum, p| sum + p.to_vec2())
                / 4.;
            if face.depth > 0.22 {
                ui.painter().text(
                    center.to_pos2(),
                    egui::Align2::CENTER_CENTER,
                    face.view.name(),
                    egui::FontId::proportional(11.),
                    Color32::WHITE,
                );
            }
        }
        let origin = rect.left_bottom() + Vec2::new(24., -22.);
        for (axis, label, color) in [
            (Vec3::X, "X", Color32::LIGHT_RED),
            (Vec3::Y, "Y", Color32::LIGHT_GREEN),
            (Vec3::Z, "Z", Color32::LIGHT_BLUE),
        ] {
            let p = camera.rotation * axis;
            let end = origin + Vec2::new(p.x, -p.y) * 19.;
            ui.painter()
                .line_segment([origin, end], Stroke::new(2., color));
            ui.painter().text(
                end,
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(11.),
                color,
            );
        }
        if !enabled {
            self.cancel();
            return None;
        }
        if response.drag_started() {
            self.previous = response
                .interact_pointer_pos()
                .map(|p| p - response.drag_delta());
        }
        if response.dragged()
            && let Some(p) = response.interact_pointer_pos()
        {
            if let Some(previous) = self.previous {
                camera.rotation =
                    (Quat::from_rotation_arc(sphere(previous, rect), sphere(p, rect))
                        * camera.rotation)
                        .normalize();
            }
            self.previous = Some(p);
        }
        if response.drag_stopped() {
            self.cancel();
        }
        if response.clicked() {
            return response.interact_pointer_pos().and_then(|p| hit(&faces, p));
        }
        None
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn face_centers_hit_visible_faces() {
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::splat(120.));
        let mut camera = Camera::default();
        for view in View::ALL {
            camera.set_view(view);
            let faces = project(camera.rotation, rect);
            assert_eq!(faces.len(), 1);
            assert_eq!(hit(&faces, rect.center()), Some(view));
            assert_eq!(hit(&faces, Pos2::new(-20., -20.)), None);
        }
    }
    #[test]
    fn arcball_is_finite_outside() {
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::splat(120.));
        assert!((sphere(Pos2::new(1000., 1000.), rect).length() - 1.).abs() < 1e-6);
    }
}
