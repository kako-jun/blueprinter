//! Mermaid → SVG bridge using the external `mmdc` (mermaid-cli) command.
//!
//! blueprinter does not parse Mermaid itself — that would duplicate years of
//! upstream work. Instead it shells out to `mmdc`, which must be on `PATH`.
//! Install via: `npm install -g @mermaid-js/mermaid-cli`.

use rand::Rng;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
pub enum RenderError {
    /// `mmdc` is not on PATH. Includes installation hint.
    MmdcNotFound,
    /// `mmdc` ran but exited non-zero. Carries stderr.
    MmdcFailed(String),
    /// Filesystem operation around the temp Mermaid/SVG files failed.
    Io(String),
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::MmdcNotFound => write!(
                f,
                "mmdc not found on PATH. Install with: npm install -g @mermaid-js/mermaid-cli"
            ),
            RenderError::MmdcFailed(msg) => write!(f, "mmdc failed: {msg}"),
            RenderError::Io(msg) => write!(f, "I/O error: {msg}"),
        }
    }
}

impl std::error::Error for RenderError {}

/// Renders a Mermaid source string to SVG by invoking `mmdc`.
///
/// The background is set to `transparent` so blueprinter's theme background
/// (chalkboard, navy, paper, …) shows through after `transform_svg` runs.
pub fn mermaid_to_svg(source: &str) -> Result<String, RenderError> {
    let (in_path, out_path) = temp_paths();
    std::fs::write(&in_path, source).map_err(|e| RenderError::Io(e.to_string()))?;

    let result = Command::new("mmdc")
        .arg("-i")
        .arg(&in_path)
        .arg("-o")
        .arg(&out_path)
        .arg("-b")
        .arg("transparent")
        .output();

    let _ = std::fs::remove_file(&in_path);

    let output = match result {
        Ok(output) => output,
        Err(e) => {
            let _ = std::fs::remove_file(&out_path);
            return Err(if e.kind() == std::io::ErrorKind::NotFound {
                RenderError::MmdcNotFound
            } else {
                RenderError::Io(e.to_string())
            });
        }
    };

    if !output.status.success() {
        let _ = std::fs::remove_file(&out_path);
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(RenderError::MmdcFailed(stderr));
    }

    let svg = std::fs::read_to_string(&out_path).map_err(|e| RenderError::Io(e.to_string()));
    let _ = std::fs::remove_file(&out_path);
    svg
}

fn temp_paths() -> (PathBuf, PathBuf) {
    let dir = std::env::temp_dir();
    let pid = std::process::id();
    let nonce: u64 = rand::thread_rng().gen();
    let stem = format!("blueprinter-mmd-{pid}-{nonce:016x}");
    (
        dir.join(format!("{stem}.mmd")),
        dir.join(format!("{stem}.svg")),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mmdc_not_found_message_includes_install_hint() {
        let err = RenderError::MmdcNotFound;
        let msg = err.to_string();
        assert!(msg.contains("npm install"));
        assert!(msg.contains("mermaid-cli"));
    }

    #[test]
    fn mmdc_failed_carries_stderr() {
        let err = RenderError::MmdcFailed("bad syntax at line 3".to_string());
        assert!(err.to_string().contains("bad syntax at line 3"));
    }

    #[test]
    fn temp_paths_have_distinct_extensions() {
        let (input, output) = temp_paths();
        assert_eq!(input.extension().and_then(|s| s.to_str()), Some("mmd"));
        assert_eq!(output.extension().and_then(|s| s.to_str()), Some("svg"));
    }

    #[test]
    fn temp_paths_are_unique_across_calls() {
        let (a, _) = temp_paths();
        let (b, _) = temp_paths();
        assert_ne!(a, b);
    }
}
