mod view_cube;
use crate::{
    app::{Action, AppState, ModelState},
    camera::{View, ViewportRect},
};
#[derive(Default)]
pub struct Ui {
    pub help: bool,
    pub background: bool,
    pub context_position: Option<egui::Pos2>,
    pub about: bool,
    pub error: Option<String>,
    pub menu_height: f32,
    pub cube: view_cube::ViewCube,
    pub viewport: ViewportRect,
}
impl Ui {
    pub fn modal(&self) -> bool {
        self.help
            || self.about
            || self.background
            || self.context_position.is_some()
            || self.error.is_some()
    }
    pub fn close_dialogs(&mut self) {
        self.help = false;
        self.background = false;
        self.context_position = None;
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
                        if ui
                            .add_enabled(model.is_some(), egui::Button::new("Export screenshot…"))
                            .clicked()
                        {
                            actions.push(Action::Screenshot);
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
                        if ui.button("Background color…").clicked() {
                            self.background = true;
                            ui.close();
                        }
                        ui.separator();
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
        if app.show_cube {
            let screen = ctx.content_rect();
            let available = (screen.height() - self.menu_height - 24.).max(0.);
            let side = 92f32.min(available).min((screen.width() - 24.).max(0.));
            if side >= 40. {
                let pos = egui::pos2(screen.left() + 12., screen.bottom() - side - 12.);
                egui::Area::new("hud".into())
                    .order(egui::Order::Background)
                    .fade_in(false)
                    .fixed_pos(pos)
                    .movable(false)
                    .interactable(false)
                    .show(ctx, |ui| {
                        ui.set_min_size(egui::vec2(side, side));
                        ui.set_max_size(egui::vec2(side, side));
                        self.cube.draw(ui, &app.camera);
                    });
            }
        }
        if app.show_zoom && model.is_some() {
            let screen = ctx.content_rect();
            if screen.height() > self.menu_height + 44. {
                egui::Area::new("zoom".into())
                    .order(egui::Order::Background)
                    .fade_in(false)
                    .anchor(egui::Align2::RIGHT_TOP, [-12., self.menu_height + 12.])
                    .movable(false)
                    .interactable(false)
                    .show(ctx, |ui| {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(format!("Zoom {:.0}%", app.camera.zoom * 100.))
                                    .color(egui::Color32::from_rgb(255, 196, 112)),
                            )
                            .wrap_mode(egui::TextWrapMode::Extend),
                        );
                    });
            }
        }
        if let Some(position) = self.context_position {
            let response = egui::Area::new("scene_context".into())
                .order(egui::Order::Foreground)
                .fixed_pos(position)
                .show(ctx, |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.set_min_width(140.);
                        if ui.button("Reset / Fit    R").clicked() {
                            actions.push(Action::Reset);
                            self.context_position = None;
                        }
                    });
                })
                .response;
            if ctx.input(|i| i.pointer.any_pressed()) && !response.contains_pointer() {
                self.context_position = None;
            }
        }
        egui::Window::new("Background color")
            .open(&mut self.background)
            .resizable(false)
            .show(ctx, |ui| {
                let old = app.background;
                ui.label("Scene background");
                ui.color_edit_button_srgb(&mut app.background);
                if ui.button("Restore default").clicked() {
                    app.background = crate::settings::DEFAULT;
                }
                ui.small("Saved automatically for future sessions.");
                if old != app.background
                    && let Err(e) = crate::settings::save(app.background)
                {
                    self.error = Some(e);
                }
            });
        egui::Window::new("Controls").open(&mut self.help).resizable(false)
            .default_width(420.).show(ctx, |ui| {
                ui.heading("Navigate the model");
                ui.add_space(8.);
                egui::Grid::new("mouse_controls").num_columns(2).spacing([28., 10.]).show(ui, |ui| {
                    for (key, description) in [("Left drag", "Orbit / rotate"), ("Right drag", "Pan"), ("Right click", "Scene menu"), ("Mouse wheel", "Zoom in / out")] {
                        ui.strong(key); ui.label(description); ui.end_row();
                    }
                });
                ui.add_space(12.); ui.separator(); ui.heading("Keyboard shortcuts"); ui.add_space(8.);
                egui::Grid::new("keyboard_controls").num_columns(2).spacing([28., 10.]).show(ui, |ui| {
                    for (key, description) in [("O / Ctrl+O", "Open STL"), ("R", "Reset view and fit model"), ("W", "Toggle wireframe"), ("F1", "Show controls"), ("Esc", "Close menu or dialog; otherwise exit")] {
                        ui.strong(key); ui.label(description); ui.end_row();
                    }
                });
                ui.add_space(12.); ui.separator(); ui.heading("View information");
                ui.label("The orientation cube shows the current view. Drag over it to navigate the scene.");
                ui.label("100% zoom is the initial fitted size. Reset restores this size.");
                ui.label("STL coordinates have no reliable physical units.");
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
                ui.hyperlink_to(
                    "GitHub · ctxinf/stl-viewer-rs",
                    "https://github.com/ctxinf/stl-viewer-rs",
                );
                if let Some(m) = model {
                    ui.label(format!(
                        "{} triangles\nCenter: {:?}\nBounds: {:?}\nRadius: {}",
                        m.triangles, m.center, m.bounds, m.radius
                    ));
                }
            });
        if let Some(error) = self.error.clone() {
            let mut open = true;
            egui::Window::new("Operation failed")
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
