pub mod scale_bar;
mod view_cube;
use crate::{
    app::{Action, AppState, ModelState},
    camera::View,
};
#[derive(Default)]
pub struct Ui {
    pub help: bool,
    pub about: bool,
    pub error: Option<String>,
    pub menu_height: f32,
    pub cube: view_cube::ViewCube,
    pub hud_rect: Option<egui::Rect>,
    pub unit: scale_bar::Unit,
}
impl Ui {
    pub fn modal(&self) -> bool {
        self.help || self.about || self.error.is_some()
    }
    pub fn close_dialogs(&mut self) {
        self.help = false;
        self.about = false;
        self.error = None;
    }
    pub fn draw(
        &mut self,
        ctx: &egui::Context,
        app: &mut AppState,
        model: Option<&ModelState>,
    ) -> Vec<Action> {
        let mut actions = Vec::new();
        let panel = egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open…    Ctrl+O").clicked() {
                        actions.push(Action::Open);
                        ui.close();
                    }
                    if ui.button("Exit").clicked() {
                        actions.push(Action::Exit);
                        ui.close();
                    }
                });
                ui.menu_button("View", |ui| {
                    if ui.button("Reset / Fit    R").clicked() {
                        actions.push(Action::Reset);
                        ui.close();
                    }
                    for view in View::ALL {
                        if ui.button(view.name()).clicked() {
                            actions.push(Action::SetView(view));
                            ui.close();
                        }
                    }
                    let mut wire = app.wire;
                    if ui.checkbox(&mut wire, "Wireframe    W").changed() {
                        actions.push(Action::ToggleWireframe);
                    }
                    ui.checkbox(&mut app.show_cube, "Orientation cube");
                    ui.checkbox(&mut app.show_scale, "Scale bar");
                    ui.menu_button("Input model units", |ui| {
                        for unit in scale_bar::Unit::ALL {
                            ui.selectable_value(&mut self.unit, unit, unit.label());
                        }
                        ui.label("Interpretation of input coordinates");
                        if let Some(mm) = self.unit.millimeters_per_unit() {
                            ui.label(format!("1 STL unit = {mm} mm"));
                        }
                    });
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("Controls    F1").clicked() {
                        self.help = true;
                        ui.close();
                    }
                    if ui.button("About").clicked() {
                        self.about = true;
                        ui.close();
                    }
                });
            });
        });
        self.menu_height = panel.response.rect.height();
        if model.is_none() {
            egui::Area::new("empty".into())
                .anchor(egui::Align2::CENTER_CENTER, [0., 0.])
                .show(ctx, |ui| {
                    ui.label("Open or drop an STL model");
                    if ui.button("Open STL…").clicked() {
                        actions.push(Action::Open);
                    }
                });
        }
        self.hud_rect = None;
        if app.show_cube {
            let screen = ctx.content_rect();
            let available = (screen.height() - self.menu_height - 64.).max(0.);
            let side = 120f32.min(available).min((screen.width() - 24.).max(0.));
            if side >= 40. {
                let pos = egui::pos2(screen.left() + 12., screen.bottom() - side - 44.);
                let enabled = !self.modal() && !ctx.is_popup_open();
                let hud = egui::Area::new("hud".into())
                    .order(egui::Order::Background)
                    .fixed_pos(pos)
                    .show(ctx, |ui| {
                        ui.set_min_size(egui::vec2(side, side));
                        ui.set_max_size(egui::vec2(side, side));
                        if let Some(view) = self.cube.draw(ui, &mut app.camera, enabled) {
                            actions.push(Action::SetView(view));
                        }
                    });
                self.hud_rect = Some(hud.response.rect);
            }
        }
        if app.show_scale
            && let Some(model) = model
        {
            let screen = ctx.content_rect();
            let p = ctx.pixels_per_point();
            let height = ((screen.height() - self.menu_height) * p).round().max(1.) as u32;
            let target = (screen.width() - 24.).clamp(0., 130.);
            if target >= 40. && screen.height() > self.menu_height + 44. {
                egui::Area::new("scale".into())
                    .order(egui::Order::Background)
                    .fixed_pos(egui::pos2(screen.left() + 12., screen.bottom() - 38.))
                    .show(ctx, |ui| {
                        ui.set_width(target);
                        scale_bar::draw(
                            ui,
                            scale_bar::calculate(
                                model.radius,
                                app.camera.vertical_span(),
                                height,
                                p,
                                target,
                            ),
                            self.unit,
                        );
                    });
            }
        }
        egui::Window::new("Controls").open(&mut self.help).resizable(false).show(ctx,|ui| {
            ui.label("Left drag: orbit · Right drag: pan · Wheel: zoom\nO / Ctrl+O: open · R: reset / fit · W: wireframe\nF1: controls · Esc: close menus / dialogs, then exit\nCube: click a face for a standard view; drag to orbit.\nSTL has no units. Default: 1 STL unit = 1 mm.\nUnits interpret input coordinates; they do not resize the model.\nThe orthographic scale bar shows screen distance, not surface measurements.");
        });
        egui::Window::new("About")
            .open(&mut self.about)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(concat!(
                    "STL Viewer ",
                    env!("CARGO_PKG_VERSION"),
                    "\nMIT License"
                ));
                if let Some(m) = model {
                    ui.label(format!(
                        "{} triangles\nCenter: {:?}\nBounds: {:?}\nRadius: {}",
                        m.triangles, m.center, m.bounds, m.radius
                    ));
                }
            });
        if let Some(error) = self.error.clone() {
            let mut open = true;
            egui::Window::new("Open failed")
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label(error);
                    if ui.button("Close").clicked() {
                        self.error = None;
                    }
                });
            if !open {
                self.error = None;
            }
        }
        actions
    }
}
