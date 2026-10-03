//! Bounded container/frame validation before any external decoder sees input.
use crate::{audio::AudioFormat, error::err, ErrorCode, SafeError};

fn corrupt() -> SafeError {
    err(ErrorCode::AudioCorrupt)
}
fn unsupported() -> SafeError {
    err(ErrorCode::AudioUnsupported)
}

pub(crate) fn validate(bytes: &[u8]) -> Result<AudioFormat, SafeError> {
    if bytes.starts_with(b"#!AMR-WB\n") {
        amr(&bytes[9..], true)?;
        return Ok(AudioFormat::Amr);
    }
    if bytes.starts_with(b"#!AMR\n") {
        amr(&bytes[6..], false)?;
        return Ok(AudioFormat::Amr);
    }
    if bytes.get(4..8) == Some(b"ftyp") {
        three_gp(bytes)?;
        return Ok(AudioFormat::ThreeGp);
    }
    if bytes.starts_with(b"ID3") || bytes.first() == Some(&0xff) {
        if bytes.len() >= 2 && bytes[0] == 0xff && bytes[1] & 0xf6 == 0xf0 {
            adts(bytes)?;
            return Ok(AudioFormat::Aac);
        }
        mp3(bytes)?;
        return Ok(AudioFormat::Mp3);
    }
    Err(unsupported())
}

fn same<T: PartialEq + Copy>(previous: &mut Option<T>, value: T) -> Result<(), SafeError> {
    if previous.is_some_and(|old| old != value) {
        return Err(corrupt());
    }
    *previous = Some(value);
    Ok(())
}

fn adts(mut bytes: &[u8]) -> Result<(), SafeError> {
    let mut attributes = None;
    while !bytes.is_empty() {
        if bytes.len() < 7 || bytes[0] != 0xff || bytes[1] & 0xf6 != 0xf0 {
            return Err(corrupt());
        }
        let frequency = (bytes[2] >> 2) & 15;
        let channels = ((bytes[2] & 1) << 2) | (bytes[3] >> 6);
        if !(3..=11).contains(&frequency) || !(1..=2).contains(&channels) {
            return Err(unsupported());
        }
        same(&mut attributes, (frequency, channels, bytes[2] >> 6))?;
        let size = (usize::from(bytes[3] & 3) << 11)
            | (usize::from(bytes[4]) << 3)
            | usize::from(bytes[5] >> 5);
        let header = if bytes[1] & 1 == 0 { 9 } else { 7 };
        if size <= header || size > bytes.len() {
            return Err(corrupt());
        }
        bytes = &bytes[size..];
    }
    if attributes.is_none() {
        return Err(err(ErrorCode::AudioEmpty));
    }
    Ok(())
}

fn amr(mut bytes: &[u8], wide: bool) -> Result<(), SafeError> {
    // Storage-frame payload bytes (including final padding bits), RFC 4867.
    const NB: [usize; 16] = [12, 13, 15, 17, 19, 20, 26, 31, 5, 6, 5, 5, 0, 0, 0, 0];
    const WB: [usize; 16] = [17, 23, 32, 36, 40, 46, 50, 58, 60, 5, 0, 0, 0, 0, 0, 0];
    if bytes.is_empty() {
        return Err(err(ErrorCode::AudioEmpty));
    }
    while !bytes.is_empty() {
        let toc = bytes[0];
        let ft = usize::from((toc >> 3) & 15);
        if toc & 0x83 != 0
            || (wide && (10..=13).contains(&ft))
            || (!wide && (12..=14).contains(&ft))
        {
            return Err(corrupt());
        }
        let size = 1 + if wide { WB[ft] } else { NB[ft] };
        if bytes.len() < size {
            return Err(corrupt());
        }
        bytes = &bytes[size..];
    }
    Ok(())
}

fn mp3(mut bytes: &[u8]) -> Result<(), SafeError> {
    if bytes.starts_with(b"ID3") {
        if bytes.len() < 10 || !(2..=4).contains(&bytes[3]) || bytes[4] == 0xff {
            return Err(corrupt());
        }
        let flags = bytes[5];
        let mask = match bytes[3] {
            2 => 0xc0,
            3 => 0xe0,
            _ => 0xf0,
        };
        if flags & !mask != 0 || bytes[6..10].iter().any(|b| b & 128 != 0) {
            return Err(corrupt());
        }
        let length = bytes[6..10]
            .iter()
            .fold(0usize, |n, b| (n << 7) | usize::from(*b));
        let end = 10
            + length
            + if bytes[3] == 4 && flags & 16 != 0 {
                10
            } else {
                0
            };
        if end > bytes.len() {
            return Err(corrupt());
        }
        bytes = &bytes[end..];
    }
    let mut attributes = None;
    while !bytes.is_empty() {
        if bytes.len() == 128 && bytes.starts_with(b"TAG") {
            bytes = &[];
            continue;
        }
        if bytes.len() < 4 || bytes[0] != 0xff || bytes[1] & 0xe0 != 0xe0 {
            return Err(corrupt());
        }
        let version = (bytes[1] >> 3) & 3;
        if version == 1 || (bytes[1] >> 1) & 3 != 1 {
            return Err(unsupported());
        }
        let bitrate = usize::from(bytes[2] >> 4);
        let frequency = usize::from((bytes[2] >> 2) & 3);
        if bitrate == 0 {
            return Err(unsupported());
        } // Free-format requires a different length contract.
        if bitrate == 15 || frequency == 3 || bytes[3] & 3 == 2 {
            return Err(corrupt());
        }
        let rate = [44100, 48000, 32000][frequency]
            / match version {
                3 => 1,
                2 => 2,
                _ => 4,
            };
        if !(8000..=48000).contains(&rate) {
            return Err(unsupported());
        }
        let channels = if bytes[3] >> 6 == 3 { 1 } else { 2 };
        same(&mut attributes, (rate, channels))?;
        let kbps = if version == 3 {
            [
                0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320,
            ][bitrate]
        } else {
            [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160][bitrate]
        };
        let size = (if version == 3 { 144000 } else { 72000 }) * kbps / rate
            + usize::from((bytes[2] >> 1) & 1);
        if size < 4 || size > bytes.len() {
            return Err(corrupt());
        }
        bytes = &bytes[size..];
    }
    if attributes.is_none() {
        return Err(err(ErrorCode::AudioEmpty));
    }
    Ok(())
}

struct BoxRef<'a> {
    kind: &'a [u8],
    body: &'a [u8],
}
fn boxes(mut bytes: &[u8]) -> Result<Vec<BoxRef<'_>>, SafeError> {
    let mut result = Vec::new();
    while !bytes.is_empty() {
        if bytes.len() < 8 {
            return Err(corrupt());
        }
        let length = u32::from_be_bytes(bytes[..4].try_into().unwrap());
        let (length, header) = match length {
            0 => (bytes.len(), 8),
            1 => {
                if bytes.len() < 16 {
                    return Err(corrupt());
                }
                (
                    usize::try_from(u64::from_be_bytes(bytes[8..16].try_into().unwrap()))
                        .map_err(|_| corrupt())?,
                    16,
                )
            }
            n => (n as usize, 8),
        };
        if length < header || length > bytes.len() {
            return Err(corrupt());
        }
        result.push(BoxRef {
            kind: &bytes[4..8],
            body: &bytes[header..length],
        });
        bytes = &bytes[length..];
    }
    Ok(result)
}

fn one<'a>(list: &'a [BoxRef<'a>], kind: &[u8]) -> Result<&'a [u8], SafeError> {
    let mut matching = list.iter().filter(|b| b.kind == kind);
    let b = matching.next().ok_or_else(corrupt)?;
    if matching.next().is_some() {
        return Err(corrupt());
    }
    Ok(b.body)
}

fn three_gp(bytes: &[u8]) -> Result<(), SafeError> {
    let top = boxes(bytes)?;
    let ftyp = one(&top, b"ftyp")?;
    if ftyp.len() < 8 || ftyp.len() % 4 != 0 {
        return Err(corrupt());
    }
    if !ftyp
        .chunks_exact(4)
        .enumerate()
        .any(|(i, b)| i != 1 && b.starts_with(b"3gp"))
    {
        return Err(unsupported());
    }
    if top.iter().any(|b| b.kind == b"moof") {
        return Err(unsupported());
    }
    let moov = boxes(one(&top, b"moov")?)?;
    let trak = boxes(one(&moov, b"trak")?)?;
    let mdia = boxes(one(&trak, b"mdia")?)?;
    let handler = one(&mdia, b"hdlr")?;
    if handler.get(8..12) != Some(b"soun") {
        return Err(unsupported());
    }
    let minf = boxes(one(&mdia, b"minf")?)?;
    let dinf = boxes(one(&minf, b"dinf")?)?;
    let dref = one(&dinf, b"dref")?;
    if dref.len() < 8 || dref[..4] != [0; 4] {
        return Err(corrupt());
    }
    let references = boxes(&dref[8..])?;
    if u32::from_be_bytes(dref[4..8].try_into().unwrap()) as usize != references.len()
        || references.is_empty()
    {
        return Err(corrupt());
    }
    for reference in &references {
        if reference.kind != b"url " || reference.body != [0, 0, 0, 1] {
            return Err(unsupported());
        }
    }
    let stbl = boxes(one(&minf, b"stbl")?)?;
    let stsd = one(&stbl, b"stsd")?;
    if stsd.len() < 8 || stsd[..8] != [0, 0, 0, 0, 0, 0, 0, 1] {
        return Err(unsupported());
    }
    let descriptions = boxes(&stsd[8..])?;
    if descriptions.len() != 1 {
        return Err(unsupported());
    }
    let sample = &descriptions[0];
    if ![b"mp4a".as_slice(), b"samr", b"sawb"].contains(&sample.kind) {
        return Err(unsupported());
    }
    if sample.body.len() < 28 {
        return Err(corrupt());
    }
    let reference = usize::from(u16::from_be_bytes(sample.body[6..8].try_into().unwrap()));
    if reference == 0 || reference > references.len() {
        return Err(corrupt());
    }
    // stsc may not switch to an undeclared sample description halfway through.
    let stsc = one(&stbl, b"stsc")?;
    if stsc.len() < 8 {
        return Err(corrupt());
    }
    let count = u32::from_be_bytes(stsc[4..8].try_into().unwrap()) as usize;
    if stsc.len() != 8 + count.checked_mul(12).ok_or_else(corrupt)? {
        return Err(corrupt());
    }
    for entry in stsc[8..].chunks_exact(12) {
        if entry[8..12] != [0, 0, 0, 1] {
            return Err(unsupported());
        }
    }
    Ok(())
}
