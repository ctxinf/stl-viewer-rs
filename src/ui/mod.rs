mod view_cube;
use crate::{
    app::{Action, AppState, ModelState},
    camera::{View, ViewportRect},
};
#[derive(Default)]
pub struct Ui {
    pub help: bool,
    pub about: bool,
    pub error: Option<String>,
    pub menu_height: f32,
    pub cube: view_cube::ViewCube,
    pub hud_rect: Option<egui::Rect>,
    pub viewport: ViewportRect,
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
        size: (u32, u32),
    ) -> Vec<Action> {
        let mut actions = Vec::new();
        let panel = egui::TopBottomPanel::top("menu")
            .exact_height(30.)
            .show(ctx, |ui| {
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
                        ui.checkbox(&mut app.show_zoom, "Zoom percentage");
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
        self.viewport = ViewportRect::below_menu(size, self.menu_height, ctx.pixels_per_point());
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
                    .fade_in(false)
                    .fixed_pos(pos)
                    .show(ctx, |ui| {
                        ui.set_min_size(egui::vec2(side, side));
                        ui.set_max_size(egui::vec2(side, side));
                        if let Some(view) = self.cube.draw(ui, &app.camera, enabled) {
                            actions.push(Action::SetView(view));
                        }
                    });
                self.hud_rect = Some(hud.response.rect);
            }
        }
        if app.show_zoom && model.is_some() {
            let screen = ctx.content_rect();
            if screen.height() > self.menu_height + 44. {
                egui::Area::new("zoom".into())
                    .order(egui::Order::Background)
                    .fade_in(false)
                    .fixed_pos(egui::pos2(screen.left() + 12., screen.bottom() - 32.))
                    .show(ctx, |ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(255, 196, 112),
                            format!("Zoom {:.0}%", app.camera.zoom * 100.),
                        );
                    });
            }
        }
        egui::Window::new("Controls").open(&mut self.help).resizable(false).show(ctx,|ui| {
            ui.label("Left drag: orbit · Right drag: pan · Wheel: zoom\nO / Ctrl+O: open · R: reset / fit · W: wireframe\nF1: controls · Esc: close menus / dialogs, then exit\nCube: click a face for a standard view. Drag and wheel work as in the scene.\nZoom: 100% is the initial fitted size; R restores 100%.\nSTL stores coordinates without reliable physical units.");
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
