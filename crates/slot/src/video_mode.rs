use std::path::Path;

use slot_gfx::WHOLE_TEXTURE;
use slot_store::{ini, Platform};

pub const VIDEO_MODE_FILE: &str = "Config/video_mode.txt";

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum VideoMode {
    #[default]
    Actual,
    Stretch,
}

impl VideoMode {
    pub fn as_str(self) -> &'static str {
        match self {
            VideoMode::Actual => "actual",
            VideoMode::Stretch => "stretch",
        }
    }

    pub fn parse(s: &str) -> Option<VideoMode> {
        match s.trim().to_ascii_lowercase().as_str() {
            "actual" => Some(VideoMode::Actual),
            "stretch" => Some(VideoMode::Stretch),
            _ => None,
        }
    }

    /// A SNES game starts at the 4:3 a TV gave it; everything else at its own pixels.
    pub fn default_for(platform: Platform) -> VideoMode {
        match platform {
            Platform::Snes => VideoMode::Stretch,
            _ => VideoMode::Actual,
        }
    }
}

pub fn video_mode_for(root: &Path, stem: &str, platform: Platform) -> VideoMode {
    ini::value(root, VIDEO_MODE_FILE, stem)
        .as_deref()
        .and_then(VideoMode::parse)
        .unwrap_or_else(|| VideoMode::default_for(platform))
}

pub fn write_video_mode(root: &Path, stem: &str, mode: VideoMode) -> std::io::Result<()> {
    ini::write(root, VIDEO_MODE_FILE, stem, mode.as_str())
}

pub fn source_rect(platform: Platform, mode: VideoMode) -> [f32; 4] {
    let (cw, _) = platform.canvas();
    let w = platform.stretched_width();
    if mode == VideoMode::Actual || w >= cw {
        return WHOLE_TEXTURE;
    }
    let x = (cw - w) / 2;
    [x as f32 / cw as f32, 0.0, w as f32 / cw as f32, 1.0]
}
