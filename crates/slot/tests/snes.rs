mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use slot::app::Phase;
use slot::session::Session;
use slot::video_mode::{video_mode_for, VideoMode};
use slot_input::{Btn, RawEvent};
use slot_retro::ButtonMask;
use slot_store::Platform;
use slot_ui::QuickRow;

fn snes_root(stems: &[&str]) -> tempfile::TempDir {
    let d = common::tmp_root_with_gb_carts(&[]);
    for stem in stems {
        std::fs::write(
            d.path().join("Games/SNES").join(format!("{stem}.sfc")),
            vec![0u8; 0x8000],
        )
        .expect("write rom");
    }
    d
}

fn session_playing(root: &Path) -> Session {
    common::clocked(root);
    let mut s = Session::boot(root.to_path_buf());
    s.feed([RawEvent::Down(Btn::A)], 16);
    s.feed([RawEvent::Up(Btn::A)], 32);
    let deadline = Instant::now() + Duration::from_secs(10);
    while !matches!(s.app().phase(), Phase::Playing { .. }) {
        assert!(Instant::now() < deadline, "the cart never seated");
        s.update(1.0 / 60.0);
        std::thread::sleep(Duration::from_millis(1));
    }
    s
}

fn pad(s: &Session) -> u16 {
    s.emu().expect("no core in the slot").input().0
}

#[test]
fn a_snes_game_starts_at_four_by_three() {
    let d = snes_root(&["Mario", "Zelda"]);
    let s = session_playing(d.path());
    assert_eq!(s.app().playing_platform(), Platform::Snes);
    assert_eq!(
        s.app().source_rect(),
        [24.0 / 336.0, 0.0, 288.0 / 336.0, 1.0],
        "a SNES game did not open on its 4:3 slice of the canvas"
    );
    assert_eq!(
        video_mode_for(d.path(), "Mario", Platform::Snes),
        VideoMode::Stretch
    );
    assert_eq!(
        video_mode_for(d.path(), "Mario", Platform::Gb),
        VideoMode::Actual,
        "a Game Boy cart with no line changed its default"
    );
}

#[test]
fn a_snes_game_gets_its_shoulders_and_x_and_y() {
    let d = snes_root(&["Mario", "Zelda"]);
    let mut s = session_playing(d.path());
    for (btn, bit) in [
        (Btn::L1, ButtonMask::L),
        (Btn::R1, ButtonMask::R),
        (Btn::X, ButtonMask::X),
        (Btn::Y, ButtonMask::Y),
    ] {
        s.feed([RawEvent::Down(btn)], 100);
        assert_ne!(pad(&s) & bit, 0, "{btn:?} did not reach the game");
        s.feed([RawEvent::Up(btn)], 120);
        assert_eq!(pad(&s) & bit, 0, "{btn:?} stuck down");
    }
    assert_eq!(
        s.app().source_rect(),
        [24.0 / 336.0, 0.0, 288.0 / 336.0, 1.0],
        "a shoulder press changed the picture instead of reaching the game"
    );
}

#[test]
fn the_triggers_reach_a_snes_game_as_turbo() {
    let d = snes_root(&["Mario", "Zelda"]);
    let mut s = session_playing(d.path());
    s.feed([RawEvent::Down(Btn::R2)], 100);
    assert_ne!(pad(&s) & ButtonMask::R2, 0, "R2 did not reach the game");
    s.feed([RawEvent::Up(Btn::R2)], 120);
    assert_eq!(pad(&s) & ButtonMask::R2, 0);
}

#[test]
fn the_snes_screen_menu_offers_the_aspect_and_no_colour_correction() {
    let d = snes_root(&["Mario", "Zelda"]);
    let s = session_playing(d.path());
    let hidden = s.app().in_game_hidden();
    assert!(hidden.contains(&QuickRow::ColourCorrection));
    assert!(hidden.contains(&QuickRow::Palette));
    assert!(!hidden.contains(&QuickRow::AspectRatio));
    assert_eq!(
        QuickRow::Shader.shown(hidden)[..2],
        [QuickRow::Shader, QuickRow::AspectRatio]
    );
}

#[test]
fn a_menu_tap_opens_the_in_game_menu_over_a_snes_game() {
    let d = snes_root(&["Mario", "Zelda"]);
    let mut s = session_playing(d.path());
    let mut now = 1_000;
    s.feed([RawEvent::Down(Btn::Menu)], now);
    now += 60;
    s.feed([RawEvent::Up(Btn::Menu)], now);
    for _ in 0..90 {
        now += 16;
        s.feed([], now);
        s.update(1.0 / 60.0);
    }
    assert_eq!(
        s.app().in_game_menu(),
        Some(QuickRow::PLAY[0]),
        "a MENU tap did not open the in-game menu, phase {:?}",
        s.app().phase()
    );
}

#[test]
fn every_menu_row_has_an_answer_while_a_snes_game_plays() {
    let d = snes_root(&["Mario", "Zelda"]);
    let s = session_playing(d.path());
    // The in-game menu reads every row's value each frame. A row that only forwards to a
    // platform's own row must not forward to itself on SNES.
    let values = QuickRow::ALL.map(|row| s.app().quick_value(row));
    assert_eq!(values[QuickRow::ColourCorrection.index()], None);
    assert!(values[QuickRow::Shader.index()].is_some());
}
