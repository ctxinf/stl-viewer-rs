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
pub struct ViewCube;
impl ViewCube {
    pub fn draw(&mut self, ui: &mut egui::Ui, camera: &Camera) {
        let (rect, _) = ui.allocate_exact_size(
            Vec2::splat(92.).min(ui.available_size()),
            egui::Sense::hover(),
        );
        let faces = project(camera.rotation, rect);
        for face in &faces {
            let color = Color32::from_gray(172 + (face.depth * 44.) as u8);
            ui.painter().add(egui::Shape::convex_polygon(
                face.points.clone(),
                color,
                Stroke::new(0.7_f32, Color32::from_gray(190)),
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
                ui.painter().arrow(origin, delta, Stroke::new(2_f32, color));
                ui.painter().text(
                    end + delta.normalized() * 8.,
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(12.),
                    color,
                );
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn standard_views_only_project_the_facing_face() {
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::splat(92.));
        let mut camera = Camera::default();
        for view in View::ALL {
            camera.set_view(view);
            let faces = project(camera.rotation, rect);
            assert_eq!(faces.len(), 1);
            assert_eq!(faces[0].view, view);
        }
    }
}
