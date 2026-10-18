use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Platform {
    #[default]
    Gba,
    Gb,
    Gbc,
    Snes,
}

impl Platform {
    pub const ALL: [Platform; 4] = [Platform::Gba, Platform::Gb, Platform::Gbc, Platform::Snes];

    pub fn name(self) -> &'static str {
        match self {
            Platform::Gba => "Game Boy Advance",
            Platform::Gb => "Game Boy",
            Platform::Gbc => "Game Boy Color",
            Platform::Snes => "Super Nintendo",
        }
    }

    pub fn dir_name(self) -> &'static str {
        match self {
            Platform::Gba => "GBA",
            Platform::Gb => "GB",
            Platform::Gbc => "GBC",
            Platform::Snes => "SNES",
        }
    }

    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Platform::Gba => &["gba"],
            Platform::Gb | Platform::Gbc => &["gb", "gbc"],
            Platform::Snes => &["sfc", "smc"],
        }
    }

    pub fn picture(self) -> (u32, u32) {
        match self {
            Platform::Gba => (240, 160),
            Platform::Gb | Platform::Gbc => (160, 144),
            Platform::Snes => (256, 224),
        }
    }

    /// The buffer the picture is centred in, always the screen's 3:2. GB and GBA share the
    /// GBA's own size. At 336 a SNES picture's square pixels make it 8:7 across the screen.
    pub fn canvas(self) -> (u32, u32) {
        match self {
            Platform::Gba | Platform::Gb | Platform::Gbc => (240, 160),
            Platform::Snes => (336, 224),
        }
    }

    /// How much of the canvas a stretched picture shows across the screen's width. A Game
    /// Boy fills it edge to edge; a SNES picture gets the 4:3 a TV gave it.
    pub fn stretched_width(self) -> u32 {
        match self {
            Platform::Snes => 288,
            _ => self.picture().0,
        }
    }

    pub fn is_game_boy(self) -> bool {
        matches!(self, Platform::Gb | Platform::Gbc)
    }

    pub fn accepts(self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| {
                self.extensions()
                    .iter()
                    .any(|k| ext.eq_ignore_ascii_case(k))
            })
    }
}
