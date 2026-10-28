use std::path::PathBuf;
pub enum Command {
    Open(Option<PathBuf>),
    Info(PathBuf),
    Screenshot {
        input: PathBuf,
        output: PathBuf,
        size: (u32, u32),
    },
    Help,
}
pub const HELP: &str = "Usage:
  stl-viewer-linux [FILE]
  stl-viewer-linux info FILE
  stl-viewer-linux screenshot FILE -o OUTPUT.png [--width 1600] [--height 1200]
  stl-viewer-linux --help

Supported models: STL, 3MF, OBJ.

info: report original bounds, dimensions, center, radius, surface area and triangle count.
screenshot: export a fitted model as PNG, using the saved background color.
Screenshot requires an OpenGL display (for headless Linux, use xvfb-run).
--info FILE remains supported.";
pub fn parse(args: impl IntoIterator<Item = std::ffi::OsString>) -> Result<Command, String> {
    let mut args = args.into_iter();
    let Some(first) = args.next() else {
        return Ok(Command::Open(None));
    };
    match first.to_str() {
        Some("--help" | "-h") => {
            if args.next().is_some() {
                return Err("Unexpected argument".into());
            }
            Ok(Command::Help)
        }
        Some("info" | "--info") => {
            let path = args.next().ok_or("info requires FILE")?;
            if args.next().is_some() {
                return Err("Unexpected info argument".into());
            }
            Ok(Command::Info(path.into()))
        }
        Some("screenshot") => {
            let input = args.next().ok_or("screenshot requires FILE")?.into();
            let mut output = None;
            let mut size = (1600, 1200);
            while let Some(option) = args.next() {
                let value = args.next().ok_or("Missing option value")?;
                match option.to_str() {
                    Some("-o" | "--output") => output = Some(PathBuf::from(value)),
                    Some("--width" | "--height") => {
                        let n: u32 = value
                            .to_str()
                            .ok_or("Invalid dimension")?
                            .parse()
                            .map_err(|_| "Invalid dimension")?;
                        if !(1..=8192).contains(&n) {
                            return Err("Dimensions must be between 1 and 8192".into());
                        }
                        if option == "--width" {
                            size.0 = n;
                        } else {
                            size.1 = n;
                        }
                    }
                    _ => return Err(format!("Unknown option: {}", option.to_string_lossy())),
                }
            }
            Ok(Command::Screenshot {
                input,
                output: output.ok_or("screenshot requires -o OUTPUT.png")?,
                size,
            })
        }
        Some(s) if s.starts_with('-') => Err(format!("Unknown option: {s}")),
        _ => {
            if args.next().is_some() {
                return Err("Expected one model file; use --help".into());
            }
            Ok(Command::Open(Some(first.into())))
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn parse_str(args: &[&str]) -> Result<Command, String> {
        parse(args.iter().map(std::ffi::OsString::from))
    }
    #[test]
    fn rejects_invalid_exports() {
        for args in [
            vec!["screenshot"],
            vec!["screenshot", "a.stl"],
            vec!["screenshot", "a.stl", "-o", "a.png", "--width", "0"],
            vec!["info", "a", "b"],
            vec!["--wat"],
        ] {
            assert!(parse_str(&args).is_err());
        }
    }
    #[test]
    fn export_options() {
        assert!(matches!(
            parse_str(&["screenshot", "a.stl", "-o", "a.png", "--width", "800"]).unwrap(),
            Command::Screenshot {
                size: (800, 1200),
                ..
            }
        ));
    }
}
