use std::collections::HashMap;
use std::path::Path;

use crate::Platform;

pub const SELECTED_CORE_FILE: &str = "Config/selected_core.txt";

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Core {
    #[default]
    Mgba,
    Gpsp,
    Snes9x,
}

impl Core {
    pub const ALL: [Core; 3] = [Core::Mgba, Core::Gpsp, Core::Snes9x];
    /// The cores a GBA game can be switched between: one socket each on the picker board.
    pub const PICKER: [Core; 2] = [Core::Mgba, Core::Gpsp];

    pub fn as_str(&self) -> &'static str {
        match self {
            Core::Mgba => "mgba",
            Core::Gpsp => "gpsp",
            Core::Snes9x => "snes9x2005_plus",
        }
    }

    /// What the core prefixes its own option keys with. snes9x2005 names its options after
    /// the core it was forked from, not after the file it ships as.
    pub fn option_prefix(&self) -> &'static str {
        match self {
            Core::Snes9x => "snes9x_2005",
            _ => self.as_str(),
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn text(self) -> &'static str {
        match self {
            Core::Mgba => "mGBA",
            Core::Gpsp => "gpSP",
            Core::Snes9x => "Snes9x",
        }
    }

    pub fn parse(s: &str) -> Option<Core> {
        match s.trim().to_ascii_lowercase().as_str() {
            "mgba" => Some(Core::Mgba),
            "gpsp" => Some(Core::Gpsp),
            "snes9x2005_plus" => Some(Core::Snes9x),
            _ => None,
        }
    }

    pub fn runs(self, platform: Platform) -> bool {
        match self {
            Core::Mgba => platform != Platform::Snes,
            Core::Gpsp => platform == Platform::Gba,
            Core::Snes9x => platform == Platform::Snes,
        }
    }

    pub fn default_for(platform: Platform) -> Core {
        match platform {
            Platform::Gba | Platform::Gb | Platform::Gbc => Core::Mgba,
            Platform::Snes => Core::Snes9x,
        }
    }
}

pub fn read_selected_cores(root: &Path) -> HashMap<String, Core> {
    crate::ini::read(root, SELECTED_CORE_FILE)
        .into_iter()
        .filter_map(|(stem, name)| Core::parse(&name).map(|core| (stem, core)))
        .collect()
}

pub fn core_for(root: &Path, stem: &str) -> Core {
    crate::ini::value(root, SELECTED_CORE_FILE, stem)
        .as_deref()
        .and_then(Core::parse)
        .unwrap_or_default()
}

pub fn core_for_platform(root: &Path, stem: &str, platform: Platform) -> Core {
    crate::ini::value(root, SELECTED_CORE_FILE, stem)
        .as_deref()
        .and_then(Core::parse)
        .filter(|core| core.runs(platform))
        .unwrap_or_else(|| Core::default_for(platform))
}

pub fn write_selected_core(root: &Path, stem: &str, core: Core) -> std::io::Result<()> {
    crate::ini::write(root, SELECTED_CORE_FILE, stem, core.as_str())
}
