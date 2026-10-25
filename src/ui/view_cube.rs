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
// Map glyph vertices onto the same orthographic face plane as the cube.
// Keeping the text in world space makes its tilt and foreshortening match the face.
fn face_label(ui: &egui::Ui, rotation: Quat, rect: Rect, view: View) {
    let normal = view.direction();
    let u = if normal.z.abs() > 0.5 {
        Vec3::X
    } else {
        Vec3::Z.cross(normal)
    };
    let v = normal.cross(u);
    let galley = ui.painter().layout_no_wrap(
        view.name().to_owned(),
        egui::FontId::proportional(12.),
        Color32::from_gray(48),
    );
    let size = galley.size();
    let fit = (42. / size.x).min(1.);
    let shape = egui::Shape::galley(Pos2::ZERO, galley, Color32::from_gray(48));
    let primitives = ui.ctx().tessellate(
        vec![egui::epaint::ClippedShape {
            clip_rect: Rect::EVERYTHING,
            shape,
        }],
        ui.ctx().pixels_per_point(),
    );
    let scale = rect.width().min(rect.height()) * 0.23;
    for primitive in primitives {
        if let egui::epaint::Primitive::Mesh(mut mesh) = primitive.primitive {
            for vertex in &mut mesh.vertices {
                let local = (vertex.pos.to_vec2() - size * 0.5) * fit / 27.;
                let p = rotation * (normal + u * local.x - v * local.y);
                vertex.pos = rect.center() + Vec2::new(p.x, -p.y) * scale;
            }
            ui.painter().add(egui::Shape::mesh(mesh));
        }
    }
}
#[derive(Default)]
pub struct ViewCube {
    previous: Option<Pos2>,
    canceled: bool,
}
impl ViewCube {
    pub fn cancel(&mut self) {
        self.previous = None;
        self.canceled = true;
    }
    pub fn draw(&mut self, ui: &mut egui::Ui, camera: &mut Camera, enabled: bool) -> Option<View> {
        let (rect, response) = ui.allocate_exact_size(
            Vec2::splat(120.).min(ui.available_size()),
            egui::Sense::click_and_drag(),
        );
        if !enabled {
            self.cancel();
        }
        if enabled && ui.input(|i| i.pointer.any_pressed()) {
            self.canceled = false;
        }
        if enabled && !self.canceled && response.drag_started() {
            self.previous = response
                .interact_pointer_pos()
                .map(|p| p - response.drag_delta());
        }
        if enabled
            && !self.canceled
            && response.dragged()
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
            self.previous = None;
        }
        let faces = project(camera.rotation, rect);
        let hover = response.hover_pos().and_then(|p| hit(&faces, p));
        for face in &faces {
            let color = if hover == Some(face.view) && enabled {
                Color32::from_rgb(174, 205, 225)
            } else {
                Color32::from_gray(172 + (face.depth * 44.) as u8)
            };
            ui.painter().add(egui::Shape::convex_polygon(
                face.points.clone(),
                color,
                Stroke::new(0.7, Color32::from_gray(190)),
            ));
            if face.depth > 0.18 {
                face_label(ui, camera.rotation, rect, face.view);
            }
        }
        let scale = rect.width().min(rect.height()) * 0.23;
        let corner = Vec3::splat(-1.);
        let project_point = |v: Vec3| {
            let p = camera.rotation * v;
            rect.center() + Vec2::new(p.x, -p.y) * scale
        };
        let origin = project_point(corner);
        for (axis, label, color) in [
            (Vec3::X, "X", Color32::from_rgb(221, 135, 135)),
            (Vec3::Y, "Y", Color32::from_rgb(125, 203, 139)),
            (Vec3::Z, "Z", Color32::from_rgb(144, 153, 224)),
        ] {
            let end = project_point(corner + axis * 2.65);
            let delta = end - origin;
            if delta.length() > 5. {
                ui.painter().arrow(origin, delta, Stroke::new(2., color));
                ui.painter().text(
                    end + delta.normalized() * 8.,
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(12.),
                    color,
                );
            }
        }
        if enabled && !self.canceled && response.clicked() {
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
