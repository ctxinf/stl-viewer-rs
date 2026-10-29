mod view_cube;
use crate::{
    app::{Action, AppState, ModelState},
    camera::{View, ViewportRect},
};
#[derive(Default)]
pub struct Ui {
    pub help: bool,
    pub colors: bool,
    pub context_position: Option<egui::Pos2>,
    pub about: bool,
    pub info: bool,
    pub error: Option<String>,
    pub menu_height: f32,
    pub cube: view_cube::ViewCube,
    pub viewport: ViewportRect,
}
impl Ui {
    pub fn modal(&self) -> bool {
        self.help
            || self.about
            || self.info
            || self.colors
            || self.context_position.is_some()
            || self.error.is_some()
    }
    pub fn close_dialogs(&mut self) {
        self.help = false;
        self.colors = false;
        self.context_position = None;
        self.about = false;
        self.info = false;
        self.error = None;
    }
    fn view_menu(&mut self, ui: &mut egui::Ui, app: &mut AppState, actions: &mut Vec<Action>) {
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
        if ui.button("Colors").clicked() {
            self.colors = true;
            ui.close();
        }
        ui.separator();
        ui.checkbox(&mut app.show_cube, "Orientation cube");
        ui.checkbox(&mut app.show_zoom, "Zoom percentage");
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
                        if ui
                            .add_enabled(model.is_some(), egui::Button::new("Info"))
                            .clicked()
                        {
                            self.info = true;
                            ui.close();
                        }
                        if ui.button("Exit").clicked() {
                            actions.push(Action::Exit);
                            ui.close();
                        }
                    });
                    ui.menu_button("View", |ui| {
                        self.view_menu(ui, app, &mut actions);
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
                    ui.label("Open or drop an STL, 3MF or OBJ model");
                    if ui.button("Open model…").clicked() {
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
            let mut open = true;
            egui::Popup::new(
                egui::Id::new("scene_context"),
                ctx.clone(),
                position,
                egui::LayerId::background(),
            )
            .kind(egui::PopupKind::Menu)
            .layout(egui::Layout::top_down_justified(egui::Align::Min))
            .style(egui::containers::menu::menu_style)
            .open_bool(&mut open)
            .show(|ui| {
                self.view_menu(ui, app, &mut actions);
            });
            if !open {
                self.context_position = None;
            }
        }
        egui::Window::new("Colors")
            .open(&mut self.colors)
            .resizable(false)
            .show(ctx, |ui| {
                for (name, label, color, default) in [
                    (
                        "background",
                        "Background",
                        &mut app.background,
                        crate::settings::DEFAULT,
                    ),
                    (
                        "model",
                        "Model",
                        &mut app.model_color,
                        crate::settings::DEFAULT_MODEL,
                    ),
                ] {
                    let old = *color;
                    ui.horizontal(|ui| {
                        ui.label(label);
                        ui.color_edit_button_srgb(color);
                        if ui.button("Restore default").clicked() {
                            *color = default;
                        }
                    });
                    if old != *color
                        && let Err(e) = crate::settings::save(name, *color)
                    {
                        self.error = Some(e);
                    }
                }
                ui.small("Saved automatically for future sessions.");
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
                    for (key, description) in [("O / Ctrl+O", "Open model"), ("R", "Reset view and fit model"), ("W", "Toggle wireframe"), ("F1", "Show controls"), ("Esc", "Close menu or dialog; otherwise exit")] {
                        ui.strong(key); ui.label(description); ui.end_row();
                    }
                });
                ui.add_space(12.); ui.separator(); ui.heading("View information");
                ui.label("The orientation cube shows the current view. Drag over it to navigate the scene.");
                ui.label("100% zoom is the initial fitted size. Reset restores this size.");
                ui.label("STL and OBJ units are unspecified; 3MF coordinates use millimeters.");
            });
        egui::Window::new("Info")
            .open(&mut self.info)
            .resizable(false)
            .show(ctx, |ui| {
                if let Some(m) = model {
                    ui.label(format!("Triangles: {}", m.triangles));
                    ui.label(format!(
                        "Load time: {:.0} ms",
                        m.load_time.as_secs_f64() * 1000.
                    ));
                    ui.separator();
                    ui.label(format!(
                        "Center: {:?}\nBounds: {:?}\nRadius: {}",
                        m.center, m.bounds, m.radius
                    ));
                }
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
            });
        if let Some(error) = self.error.clone() {
            let mut open = true;
            egui::Window::new("Operation failed")
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label(&error);
                    if ui.button("Copy error").clicked() {
                        ctx.copy_text(error.clone());
                    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_menu_stays_in_window_and_closes_after_reset() {
        let ctx = egui::Context::default();
        let mut ui = Ui {
            context_position: Some(egui::pos2(790., 590.)),
            ..Default::default()
        };
        let mut app = AppState::default();
        let mut frame = |events| {
            let mut actions = Vec::new();
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800., 600.),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| actions = ui.draw(ctx, &mut app, None, (800, 600)),
            );
            actions
        };
        // Let egui measure the menu before checking placement and clicking it.
        for _ in 0..3 {
            frame(Vec::new());
        }
        let rect = ctx
            .read_response(egui::Id::new("scene_context"))
            .unwrap()
            .rect;
        assert!(rect.right() <= 800. && rect.bottom() <= 600., "{rect:?}");
        assert!(rect.height() > 200., "The full View menu must be present");
        let pos = rect.min + egui::vec2(30., 14.);
        frame(vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
        ]);
        let actions = frame(vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        }]);
        assert!(actions.iter().any(|action| matches!(action, Action::Reset)));
        assert!(ui.context_position.is_none());
    }
}
