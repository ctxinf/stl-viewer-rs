use glium::winit::event_loop::EventLoopProxy;
use std::{ffi::OsString, path::PathBuf, time::Instant};

pub enum Result {
    Open(Option<PathBuf>),
    Screenshot(Option<PathBuf>),
}

pub enum UserEvent {
    Repaint(Instant),
    Dialog(Result),
}

pub fn show(screenshot_name: Option<OsString>, proxy: EventLoopProxy<UserEvent>) {
    let show = move || {
        let result = if let Some(name) = screenshot_name {
            Result::Screenshot(
                rfd::FileDialog::new()
                    .add_filter("PNG image", &["png"])
                    .set_file_name(name.to_string_lossy())
                    .save_file(),
            )
        } else {
            Result::Open(
                rfd::FileDialog::new()
                    .add_filter("STL model", &["stl", "STL"])
                    .pick_file(),
            )
        };
        let _ = proxy.send_event(UserEvent::Dialog(result));
    };
    // Portal startup and user interaction may take arbitrarily long. Keep the
    // Linux window loop running so the compositor can still ping and repaint it.
    #[cfg(target_os = "linux")]
    run_dialog(show);
    // Preserve native dialog threading requirements on the other platforms.
    #[cfg(not(target_os = "linux"))]
    show();
}

#[cfg(target_os = "linux")]
fn run_dialog(show: impl FnOnce() + Send + 'static) {
    std::thread::spawn(show);
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn dialog_wait_does_not_block_caller() {
        let caller = std::thread::current().id();
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let (finished_tx, finished_rx) = mpsc::channel();
        run_dialog(move || {
            started_tx.send(std::thread::current().id()).unwrap();
            // A bounded wait also makes a synchronous regression fail instead of hang.
            let released = release_rx.recv_timeout(Duration::from_secs(2)).is_ok();
            finished_tx.send(released).unwrap();
        });
        assert_ne!(
            started_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
            caller
        );
        assert!(matches!(
            finished_rx.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));
        release_tx.send(()).unwrap();
        assert!(finished_rx.recv_timeout(Duration::from_secs(5)).unwrap());
    }
}
