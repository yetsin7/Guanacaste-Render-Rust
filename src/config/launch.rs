//! Developer launch options: `--screen=<slug>` opens a given screen at start-up and `--size=<W>x<H>` opens a
//! fixed-size window in interface points (useful to compare against the prototype screenshots); `--zoom=<f>`
//! scales the whole interface. The `GR_SCREEN` environment variable
//! is an alternative to `--screen`.

use crate::models::navigation::Screen;

/// Options read from the command line.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LaunchOptions {
    /// Screen to show first.
    pub screen: Option<Screen>,
    /// Fixed inner window size in logical points, when given.
    pub size: Option<(f32, f32)>,
    /// Interface zoom factor (e.g. `0.7` to see tall screens whole in a screenshot).
    pub zoom: Option<f32>,
}

impl LaunchOptions {
    /// Parses the given arguments (without the program name).
    pub fn parse<I, S>(args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut options = Self::default();
        for arg in args {
            let arg = arg.as_ref();
            if let Some(slug) = arg.strip_prefix("--screen=") {
                options.screen = Screen::from_slug(slug);
            } else if let Some(size) = arg.strip_prefix("--size=") {
                options.size = parse_size(size);
            } else if let Some(zoom) = arg.strip_prefix("--zoom=") {
                options.zoom = zoom.parse().ok();
            }
        }
        options
    }

    /// Reads the process arguments and the `GR_SCREEN` environment variable.
    pub fn from_environment() -> Self {
        let mut options = Self::parse(std::env::args().skip(1));
        if options.screen.is_none() {
            if let Ok(slug) = std::env::var("GR_SCREEN") {
                options.screen = Screen::from_slug(&slug);
            }
        }
        options
    }
}

/// Parses `"1333x1067"` into a size.
fn parse_size(text: &str) -> Option<(f32, f32)> {
    let (width, height) = text.split_once('x')?;
    Some((width.parse().ok()?, height.parse().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_screen_and_size() {
        let options = LaunchOptions::parse(["--screen=library", "--size=1333x1067"]);
        assert_eq!(options.screen, Some(Screen::Library));
        assert_eq!(options.size, Some((1333.0, 1067.0)));
        assert_eq!(LaunchOptions::parse(["--zoom=0.7"]).zoom, Some(0.7));
    }

    #[test]
    fn ignores_unknown_values() {
        let options = LaunchOptions::parse(["--screen=nope", "--size=abc", "--other"]);
        assert_eq!(options, LaunchOptions::default());
    }
}
