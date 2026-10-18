use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Where the internal header sits for LoROM, HiROM and ExHiROM carts.
const HEADERS: [u64; 3] = [0x7fc0, 0xffc0, 0x40ffc0];
/// A dump from a copier carries 512 bytes of its own in front of the ROM.
const COPIER: u64 = 512;
const HEADER_LEN: usize = 0x20;
const TITLE_LEN: usize = 21;
const MAP_OFF: usize = 0x15;
const COMPLEMENT_OFF: usize = 0x1c;

/// The internal title of a SNES ROM, from whichever header candidate looks most real.
pub fn title(rom: &Path) -> Option<String> {
    let mut f = File::open(rom).ok()?;
    let len = f.metadata().ok()?.len();
    let skip = if len % 1024 == COPIER { COPIER } else { 0 };
    HEADERS
        .iter()
        .filter_map(|&at| {
            let mut buf = [0u8; HEADER_LEN];
            f.seek(SeekFrom::Start(skip + at)).ok()?;
            f.read_exact(&mut buf).ok()?;
            Some((score(&buf, at), buf))
        })
        .filter(|(score, _)| *score > 0)
        .max_by_key(|(score, _)| *score)
        .and_then(|(_, buf)| text(&buf[..TITLE_LEN]))
}

fn score(header: &[u8; HEADER_LEN], at: u64) -> u32 {
    let mut score = 0;
    let checksum = u16::from_le_bytes([header[COMPLEMENT_OFF + 2], header[COMPLEMENT_OFF + 3]]);
    let complement = u16::from_le_bytes([header[COMPLEMENT_OFF], header[COMPLEMENT_OFF + 1]]);
    if checksum ^ complement == 0xffff {
        score += 4;
    }
    let map = header[MAP_OFF] & 0xef;
    let hirom = at != HEADERS[0];
    if (map & 0x01 == 1) == hirom && (0x20..=0x25).contains(&map) {
        score += 2;
    }
    if header[..TITLE_LEN].iter().all(|b| (0x20..0x7f).contains(b)) {
        score += 1;
    }
    score
}

fn text(bytes: &[u8]) -> Option<String> {
    let text: String = bytes
        .iter()
        .map(|&b| {
            if (0x20..0x7f).contains(&b) {
                b as char
            } else {
                ' '
            }
        })
        .collect();
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    (!text.is_empty()).then_some(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rom(size: usize, at: usize, title: &str, map: u8) -> Vec<u8> {
        let mut rom = vec![0xffu8; size];
        let header = &mut rom[at..at + HEADER_LEN];
        header[..TITLE_LEN].fill(b' ');
        header[..title.len()].copy_from_slice(title.as_bytes());
        header[MAP_OFF] = map;
        header[COMPLEMENT_OFF..COMPLEMENT_OFF + 4].copy_from_slice(&[0x34, 0x12, 0xcb, 0xed]);
        rom
    }

    fn title_of(bytes: &[u8]) -> Option<String> {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("game.sfc");
        std::fs::write(&path, bytes).unwrap();
        title(&path)
    }

    #[test]
    fn a_lorom_title_is_read_at_7fc0() {
        let bytes = rom(0x80000, 0x7fc0, "SUPER MARIOWORLD", 0x20);
        assert_eq!(title_of(&bytes).as_deref(), Some("SUPER MARIOWORLD"));
    }

    #[test]
    fn a_hirom_title_is_read_at_ffc0() {
        let bytes = rom(0x100000, 0xffc0, "SECRET OF MANA", 0x31);
        assert_eq!(title_of(&bytes).as_deref(), Some("SECRET OF MANA"));
    }

    #[test]
    fn a_copier_header_is_skipped() {
        let mut bytes = vec![0u8; 512];
        bytes.extend(rom(0x80000, 0x7fc0, "ZELDA", 0x20));
        assert_eq!(title_of(&bytes).as_deref(), Some("ZELDA"));
    }

    #[test]
    fn a_rom_without_a_header_has_no_title() {
        assert_eq!(title_of(&vec![0u8; 0x8000]), None);
    }
}
