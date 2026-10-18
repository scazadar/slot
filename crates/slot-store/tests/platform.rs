use slot_store::Platform;

#[test]
fn every_platform_has_a_directory() {
    let names: Vec<&str> = Platform::ALL.iter().map(|p| p.dir_name()).collect();
    assert_eq!(names, vec!["GBA", "GB", "GBC", "SNES"]);
}

#[test]
fn every_platform_is_a_shelf_of_its_own() {
    assert_eq!(
        Platform::ALL,
        [Platform::Gba, Platform::Gb, Platform::Gbc, Platform::Snes]
    );
    for (i, p) in Platform::ALL.iter().enumerate() {
        assert!(
            !Platform::ALL[..i].contains(p),
            "{p:?} is named twice in the ring, so two shelves would hold one platform"
        );
    }
}

#[test]
fn each_platform_takes_only_its_own_extensions() {
    assert!(Platform::Gba.accepts(std::path::Path::new("Metroid Fusion.gba")));
    assert!(!Platform::Gba.accepts(std::path::Path::new("Tetris.gb")));
    assert!(Platform::Gb.accepts(std::path::Path::new("Tetris.gb")));
    assert!(Platform::Gb.accepts(std::path::Path::new("Tetris.gbc")));
    assert!(!Platform::Gb.accepts(std::path::Path::new("Metroid Fusion.gba")));
    assert!(Platform::Gba.accepts(std::path::Path::new("Shrek.GBA")));
    assert!(Platform::Snes.accepts(std::path::Path::new("Super Metroid.sfc")));
    assert!(Platform::Snes.accepts(std::path::Path::new("Chrono Trigger.SMC")));
    assert!(!Platform::Snes.accepts(std::path::Path::new("Tetris.gb")));
}

#[test]
fn every_canvas_has_the_screens_shape() {
    for p in Platform::ALL {
        let (w, h) = p.canvas();
        assert_eq!(w * 2, h * 3, "{p:?}'s canvas is not 3:2");
        let (pw, ph) = p.picture();
        assert!(
            pw <= w && ph <= h,
            "{p:?}'s picture does not fit its canvas"
        );
    }
}

/// The slice of the canvas shown fills the 3:2 screen, so a 256 wide picture in a slice `w`
/// wide comes out 1.5 * 256 / w as wide as it is tall.
fn shown_aspect(slice: u32) -> f32 {
    1.5 * Platform::Snes.picture().0 as f32 / slice as f32
}

#[test]
fn a_snes_picture_is_four_by_three_stretched_and_eight_by_seven_actual() {
    assert!((shown_aspect(Platform::Snes.stretched_width()) - 4.0 / 3.0).abs() < 1e-6);
    assert!((shown_aspect(Platform::Snes.canvas().0) - 8.0 / 7.0).abs() < 1e-6);
}
