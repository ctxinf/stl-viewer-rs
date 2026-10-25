mod app;
mod camera;
mod cli;
mod export;
mod mesh;
mod renderer;
mod settings;
mod ui;
use app::{Action, AppState};
use mesh::Vertex;
glium::implement_vertex!(Vertex, position);

use glium::{
    Surface,
    winit::{
        self,
        event::{ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent},
        keyboard::{KeyCode, PhysicalKey},
    },
};
use std::{path::PathBuf, time::Instant};

fn open_mesh(path: &std::path::Path) -> Result<mesh::Mesh, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    // Read-only mapping is dropped after parsing. The source must not be
    // modified or truncated concurrently while opening it.
    let data = unsafe { memmap2::Mmap::map(&file) }.map_err(|e| e.to_string())?;
    mesh::Mesh::parse(&data)
}

fn main() {
    if let Err(e) = run() {
        eprintln!("STL Viewer: {e}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let command = cli::parse(std::env::args_os().skip(1))?;
    let (first, screenshot) = match command {
        cli::Command::Help => {
            println!("{}", cli::HELP);
            return Ok(());
        }
        cli::Command::Info(path) => {
            let start = Instant::now();
            let mesh = open_mesh(&path)?;
            let dimensions: [f64; 3] =
                std::array::from_fn(|i| mesh.bounds[1][i] - mesh.bounds[0][i]);
            let area: f64 = mesh
                .vertices
                .as_chunks::<3>()
                .0
                .iter()
                .map(|t| {
                    let a = glam::DVec3::from_array(t[0].position.map(f64::from));
                    let b = glam::DVec3::from_array(t[1].position.map(f64::from));
                    let c = glam::DVec3::from_array(t[2].position.map(f64::from));
                    (b - a).cross(c - a).length() * 0.5 * mesh.radius.powi(2)
                })
                .sum();
            println!(
                "File: {}\nBytes: {}\nTriangles: {}\nVertices (with duplicates): {}\nBounds min: {:?}\nBounds max: {:?}\nDimensions: {:?}\nCenter: {:?}\nRadius: {}\nSurface area: {}\nUnits: unspecified (STL)\nParse time: {:.3} ms",
                path.display(),
                std::fs::metadata(&path)?.len(),
                mesh.vertices.len() / 3,
                mesh.vertices.len(),
                mesh.bounds[0],
                mesh.bounds[1],
                dimensions,
                mesh.center,
                mesh.radius,
                area,
                start.elapsed().as_secs_f64() * 1000.
            );
            return Ok(());
        }
        cli::Command::Screenshot {
            input,
            output,
            size,
        } => (Some(input), Some((output, size))),
        cli::Command::Open(path) => (path, None),
    };
    // Parse exports before creating a graphics context so file errors work without a display.
    let export_mesh = if screenshot.is_some() {
        Some(open_mesh(first.as_ref().unwrap())?)
    } else {
        None
    };
    let event_loop = winit::event_loop::EventLoop::<Instant>::with_user_event().build()?;
    let mut icon_reader = png::Decoder::new(std::io::Cursor::new(include_bytes!(
        "../packaging/stl-viewer-linux.png"
    )))
    .read_info()?;
    let mut icon_data = vec![
        0;
        icon_reader
            .output_buffer_size()
            .ok_or("Invalid icon size")?
    ];
    let icon_info = icon_reader.next_frame(&mut icon_data)?;
    icon_data.truncate(icon_info.buffer_size());
    let icon = winit::window::Icon::from_rgba(icon_data, icon_info.width, icon_info.height)?;
    let attributes = winit::window::Window::default_attributes()
        .with_window_icon(Some(icon))
        .with_visible(screenshot.is_none());
    let attributes = winit::platform::x11::WindowAttributesExtX11::with_name(
        attributes,
        "stl-viewer-linux",
        "stl-viewer-linux",
    );
    let attributes = winit::platform::wayland::WindowAttributesExtWayland::with_name(
        attributes,
        "stl-viewer-linux",
        "stl-viewer-linux",
    );
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .set_window_builder(attributes)
        .with_title("STL Viewer — O: open / drop an STL")
        .with_inner_size(1000, 760)
        .build(&event_loop);
    let renderer = renderer::Renderer::new(&display)?;
    if let Some((path, size)) = screenshot {
        let mesh = export_mesh.unwrap();
        let buffer = glium::VertexBuffer::immutable(&display, &mesh.vertices)?;
        export::screenshot(
            &display,
            &renderer,
            &buffer,
            &AppState::default(),
            size,
            &path,
        )?;
        println!("Saved {} ({} × {})", path.display(), size.0, size.1);
        return Ok(());
    }
    let mut buffer = None;
    let mut app = AppState::default();
    let mut ui = ui::Ui::default();
    let mut egui =
        egui_glium::EguiGlium::new(egui::ViewportId::ROOT, &display, &window, &event_loop);
    let proxy = event_loop.create_proxy();
    egui.egui_ctx().set_request_repaint_callback(move |info| {
        if let Some(deadline) = Instant::now().checked_add(info.delay) {
            let _ = proxy.send_event(deadline);
        }
    });
    let mut repaint_at: Option<Instant> = None;
    let mut modifiers = winit::keyboard::ModifiersState::empty();
    let mut cursor = None;
    let mut left = false;
    let mut right = false;
    let mut right_start: Option<(f64, f64)> = None;
    let load = |path: PathBuf| -> Result<app::ModelState, String> {
        let start = Instant::now();
        let mesh = open_mesh(&path)?;
        let buffer =
            glium::VertexBuffer::immutable(&display, &mesh.vertices).map_err(|e| e.to_string())?;
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        window.set_title(&format!(
            "{name} — {} triangles — {:.0} ms | O open · R reset · W wireframe",
            mesh.vertices.len() / 3,
            start.elapsed().as_secs_f64() * 1000.
        ));
        let mut screenshot_name = path.file_stem().unwrap_or_default().to_os_string();
        screenshot_name.push("-screenshot.png");
        Ok(app::ModelState {
            screenshot_name,
            buffer,
            triangles: mesh.vertices.len() / 3,
            center: mesh.center,
            radius: mesh.radius,
            bounds: mesh.bounds,
        })
    };
    if let Some(path) = first {
        match load(path) {
            Ok(b) => buffer = Some(b),
            Err(e) => {
                window.set_title(&format!("STL Viewer — {e} — O to open"));
                ui.error = Some(e.clone());
                eprintln!("{e}");
            }
        }
    }
    #[allow(deprecated)]
    event_loop.run(|event, target| {
        target.set_control_flow(winit::event_loop::ControlFlow::Wait);
        if let Some(deadline) = repaint_at {
            if Instant::now() >= deadline {
                window.request_redraw();
                repaint_at = None;
            } else {
                target.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(deadline));
            }
        }
        if let Event::UserEvent(deadline) = event {
            repaint_at = Some(repaint_at.map_or(deadline, |old| old.min(deadline)));
            if deadline <= Instant::now() {
                window.request_redraw();
            } else {
                target.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                    repaint_at.unwrap(),
                ));
            }
            return;
        }
        let Event::WindowEvent { event, .. } = event else {
            return;
        };
        let response = egui.on_event(&window, &event);
        if response.repaint && !matches!(event, WindowEvent::RedrawRequested) {
            window.request_redraw();
        }
        let blocked = ui.modal() || egui.egui_ctx().is_popup_open() || response.consumed;
        let mut actions = Vec::new();
        let mut open = None;
        match event {
            WindowEvent::CloseRequested => target.exit(),
            WindowEvent::DroppedFile(path) => open = Some(path),
            WindowEvent::Resized(size) if size.width > 0 && size.height > 0 => {
                display.resize(size.into());
                window.request_redraw();
            }
            WindowEvent::ModifiersChanged(value) => modifiers = value.state(),
            WindowEvent::MouseInput { state, button, .. } => {
                if state == ElementState::Pressed && blocked {
                    return;
                }
                if button == MouseButton::Left {
                    left = state == ElementState::Pressed;
                }
                if button == MouseButton::Right {
                    right = state == ElementState::Pressed;
                    if right {
                        right_start = cursor;
                    } else if let (Some(start), Some(end)) = (right_start.take(), cursor)
                        && (start.0 - end.0).hypot(start.1 - end.1) < 5.
                    {
                        ui.context_position = Some(egui::pos2(
                            end.0 as f32 / window.scale_factor() as f32,
                            end.1 as f32 / window.scale_factor() as f32,
                        ));
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::CursorLeft { .. } => {
                cursor = None;
                right_start = None;
                left = false;
                right = false;
            }
            WindowEvent::Focused(false) => {
                cursor = None;
                right_start = None;
                left = false;
                right = false;
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some((x, y)) = cursor {
                    let dx = (position.x - x) as f32;
                    let dy = (position.y - y) as f32;
                    if left {
                        app.camera.orbit(dx, dy);
                    }
                    if right {
                        if let Some((sx, sy)) = right_start
                            && (position.x - sx).hypot(position.y - sy) >= 5.
                        {
                            right_start = None;
                        }
                        let h = ui.viewport.height.max(1) as f32;
                        app.camera.pan[0] += dx / h * app.camera.vertical_span() as f32;
                        app.camera.pan[1] -= dy / h * app.camera.vertical_span() as f32;
                    }
                    if left || right {
                        window.request_redraw();
                    }
                }
                cursor = Some((position.x, position.y));
            }
            WindowEvent::MouseWheel { delta, .. } if !blocked => {
                let d = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 / 40.,
                };
                app.camera.zoom = (app.camera.zoom * (d * 0.12).exp()).clamp(0.02, 100.);
                window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed && !event.repeat =>
            {
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::Escape) => {
                        if ui.modal() {
                            ui.close_dialogs();
                        } else if egui.egui_ctx().is_popup_open() {
                            egui::Popup::close_all(egui.egui_ctx());
                        } else {
                            actions.push(Action::Exit);
                        }
                    }
                    PhysicalKey::Code(KeyCode::F1) if !egui.egui_ctx().wants_keyboard_input() => {
                        ui.help = true
                    }
                    PhysicalKey::Code(key)
                        if !ui.modal()
                            && !egui.egui_ctx().is_popup_open()
                            && !egui.egui_ctx().wants_keyboard_input() =>
                    {
                        match key {
                            KeyCode::KeyR => actions.push(Action::Reset),
                            KeyCode::KeyW => actions.push(Action::ToggleWireframe),
                            KeyCode::KeyO if modifiers.is_empty() || modifiers.control_key() => {
                                actions.push(Action::Open)
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let size = window.inner_size();
                if size.width == 0 || size.height == 0 {
                    return;
                }
                egui.run(&window, |ctx| {
                    actions.extend(ui.draw(
                        ctx,
                        &mut app,
                        buffer.as_ref(),
                        (size.width, size.height),
                    ));
                });

                let mut frame = display.draw();
                let c = app.background.map(|v| v as f32 / 255.);
                frame.clear_color_and_depth((c[0], c[1], c[2], 1.), 1.);
                if let Some(buffer) = &buffer {
                    let viewport = ui.viewport;
                    let matrix = app.camera.matrix(viewport);
                    if let Err(e) =
                        renderer.draw(&mut frame, &buffer.buffer, matrix, viewport, app.wire)
                    {
                        eprintln!("Render failed: {e}");
                        target.exit();
                    }
                }
                egui.paint(&display, &mut frame);
                if let Err(e) = frame.finish() {
                    eprintln!("Present failed: {e}");
                    target.exit();
                }
            }
            _ => {}
        }
        for action in actions {
            match action {
                Action::Open => {
                    open = rfd::FileDialog::new()
                        .add_filter("STL model", &["stl", "STL"])
                        .pick_file()
                }
                Action::Screenshot => {
                    if let Some(model) = &buffer
                        && let Some(path) = rfd::FileDialog::new()
                            .add_filter("PNG image", &["png"])
                            .set_file_name(model.screenshot_name.to_string_lossy())
                            .save_file()
                        && let Err(e) = export::screenshot(
                            &display,
                            &renderer,
                            &model.buffer,
                            &app,
                            (ui.viewport.width, ui.viewport.height),
                            &path,
                        )
                    {
                        ui.error = Some(e.to_string());
                    }
                }
                Action::Exit => target.exit(),
                action => app.apply(action),
            }
            window.request_redraw();
        }
        if let Some(path) = open {
            match load(path) {
                Ok(b) => {
                    buffer = Some(b);
                    app.apply(Action::Reset);
                }
                Err(e) => {
                    window.set_title(&format!("STL Viewer — {e} — O to retry"));
                    ui.error = Some(e.clone());
                    eprintln!("{e}");
                }
            }
            window.request_redraw();
        }
    })?;
    Ok(())
}
