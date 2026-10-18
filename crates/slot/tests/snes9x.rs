mod common;

use slot_retro::{ButtonMask, RetroCore};
use slot_store::{Core, Platform};

fn dylib() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor")
        .join(slot::core::dylib_name(Core::Snes9x))
}

/// A 32 KiB LoROM that turns the backdrop white, switches the screen on and spins. Each NMI
/// vector points at an RTI.
fn snes_rom() -> Vec<u8> {
    const CODE: [u8; 31] = [
        0x78, // sei
        0x18, // clc
        0xfb, // xce: native mode
        0xe2, 0x20, // sep #$20: 8-bit A
        0xa9, 0x00, 0x8d, 0x21, 0x21, // CGADD = 0
        0xa9, 0xff, 0x8d, 0x22, 0x21, // colour 0, low byte
        0xa9, 0x7f, 0x8d, 0x22, 0x21, // colour 0, high byte: white
        0xa9, 0x0f, 0x8d, 0x00, 0x21, // INIDISP: screen on, full brightness
        0x80, 0xfe, // bra .
        0x40, // rti
        0xea, 0xea, 0xea,
    ];
    const RTI: u16 = 0x8000 + 27;
    let mut rom = vec![0u8; 0x8000];
    rom[..CODE.len()].copy_from_slice(&CODE);
    let header = &mut rom[0x7fc0..];
    header[..21].copy_from_slice(b"SLOT TEST            ");
    header[0x15] = 0x20; // LoROM, slow
    header[0x17] = 0x05; // 32 KiB
    header[0x19] = 0x01; // North America
    for vector in (0x7fe4..0x7ffc).step_by(2) {
        rom[vector..vector + 2].copy_from_slice(&RTI.to_le_bytes());
    }
    rom[0x7ffc..0x7ffe].copy_from_slice(&0x8000u16.to_le_bytes());
    rom[0x7fdc..0x7fe0].copy_from_slice(&[0xff, 0xff, 0x00, 0x00]);
    let sum = rom.iter().fold(0u16, |a, b| a.wrapping_add(*b as u16));
    rom[0x7fdc..0x7fde].copy_from_slice(&(sum ^ 0xffff).to_le_bytes());
    rom[0x7fde..0x7fe0].copy_from_slice(&sum.to_le_bytes());
    rom
}

fn booted() -> Option<(slot_retro::LibretroCore, tempfile::TempDir)> {
    let path = dylib();
    if !path.exists() {
        eprintln!("no snes9x2005_plus core in vendor/, skipping");
        return None;
    }
    let dir = tempfile::tempdir().unwrap();
    let rom = dir.path().join("test.sfc");
    std::fs::write(&rom, snes_rom()).unwrap();
    let mut core = slot_retro::LibretroCore::open(&path).expect("open snes9x");
    let (w, h) = Platform::Snes.canvas();
    core.set_canvas(w, h);
    slot::core::apply_core_options(
        &mut core,
        Core::Snes9x,
        Platform::Snes,
        "",
        false,
        false,
        None,
    );
    core.load(&rom).expect("snes9x loads the test rom");
    Some((core, dir))
}

#[test]
fn the_snes_core_resolves_to_its_own_dylib() {
    assert!(slot::core::dylib_name(Core::Snes9x).starts_with("snes9x2005_plus_libretro."));
    assert_eq!(
        slot_store::core_for_platform(std::path::Path::new("."), "x", Platform::Snes),
        Core::Snes9x
    );
}

#[test]
fn snes9x_is_told_to_skip_frames_under_its_own_option_name() {
    let _g = common::core_lock();
    let Some((core, _dir)) = booted() else {
        return;
    };
    assert!(
        core.declared_options()
            .contains_key("snes9x_2005_frameskip"),
        "snes9x2005 renamed its frameskip option"
    );
    assert_eq!(
        core.option("snes9x_2005_frameskip").as_deref(),
        Some("auto")
    );
}

#[test]
fn a_snes_frame_lands_centred_in_the_snes_canvas() {
    let _g = common::core_lock();
    let Some((mut core, _dir)) = booted() else {
        return;
    };
    for _ in 0..10 {
        core.run_frame(ButtonMask::default());
    }
    let av = core.av_info();
    assert!(
        (av.fps - 60.0).abs() < 0.2,
        "an NTSC game runs at {} fps",
        av.fps
    );
    assert_eq!(av.sample_rate, 32040.0);
    let video = core.video_xrgb8888();
    assert_eq!(video.len(), 336 * 224 * 4);
    let lit = |x: usize, y: usize| video[(y * 336 + x) * 4 + 1] > 0xc0;
    assert!(
        lit(40, 0) && lit(295, 223),
        "the white backdrop does not fill x 40..296"
    );
    assert!(
        !lit(39, 100) && !lit(296, 100),
        "the side bars are not black"
    );
    assert!(
        !core.take_audio().is_empty(),
        "the core made no sound samples"
    );
}

#[test]
fn a_snes_state_round_trips() {
    let _g = common::core_lock();
    let Some((mut core, _dir)) = booted() else {
        return;
    };
    core.run_frame(ButtonMask::default());
    let state = core.serialize().expect("snes9x serializes");
    assert!(!state.is_empty());
    core.run_frame(ButtonMask::default());
    core.unserialize(&state)
        .expect("snes9x takes its own state back");
}
