extern crate alloc;
use alloc::vec;
use encoding_rs::{CoderResult, Decoder, Encoder, Encoding};

pub struct IconvHandle {
    pub from_type: u8,
    pub to_type: u8,
    pub decoder: Option<Decoder>,
    pub encoder: Option<Encoder>,
    pub state: u32,
    pub is_stateful: bool,
}

impl core::fmt::Debug for IconvHandle {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IconvHandle")
            .field("from_type", &self.from_type)
            .field("to_type", &self.to_type)
            .field("state", &self.state)
            .field("is_stateful", &self.is_stateful)
            .field("has_decoder", &self.decoder.is_some())
            .field("has_encoder", &self.encoder.is_some())
            .finish()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum IconvError {
    IllegalSequence,
    IncompleteInput,
    InvalidEncoding,
    OutputOverflow,
}

pub const ENCODING_UTF_8: u8 = 0;
pub const ENCODING_US_ASCII: u8 = 1;
pub const ENCODING_WCHAR_T: u8 = 2;
pub const ENCODING_UCS2BE: u8 = 3;
pub const ENCODING_UCS2LE: u8 = 4;
pub const ENCODING_UTF_16BE: u8 = 5;
pub const ENCODING_UTF_16LE: u8 = 6;
pub const ENCODING_UCS2: u8 = 7;
pub const ENCODING_UTF_16: u8 = 8;
pub const ENCODING_UTF_32: u8 = 9;
pub const ENCODING_SHIFT_JIS: u8 = 10;
pub const ENCODING_EUC_JP: u8 = 11;
pub const ENCODING_ISO2022_JP: u8 = 12;
pub const ENCODING_GB2312: u8 = 13;
pub const ENCODING_GBK: u8 = 14;
pub const ENCODING_GB18030: u8 = 15;
pub const ENCODING_BIG5: u8 = 16;
pub const ENCODING_EUC_KR: u8 = 17;
pub const ENCODING_UTF_32BE: u8 = 18;
pub const ENCODING_UTF_32LE: u8 = 19;
pub const ENCODING_COUNT: u8 = 20;

pub fn resolve_encoding(enc_type: u8) -> Option<&'static Encoding> {
    match enc_type {
        ENCODING_UTF_8 => Some(encoding_rs::UTF_8),
        ENCODING_SHIFT_JIS => Some(encoding_rs::SHIFT_JIS),
        ENCODING_EUC_JP => Some(encoding_rs::EUC_JP),
        ENCODING_GBK => Some(encoding_rs::GBK),
        ENCODING_GB2312 => Some(encoding_rs::GBK),
        ENCODING_GB18030 => Some(encoding_rs::GB18030),
        ENCODING_BIG5 => Some(encoding_rs::BIG5),
        ENCODING_EUC_KR => Some(encoding_rs::EUC_KR),
        _ => None,
    }
}

fn encoding_to_type(enc: &'static Encoding) -> Option<u8> {
    if enc == encoding_rs::UTF_8 { return Some(ENCODING_UTF_8); }
    if enc == encoding_rs::SHIFT_JIS { return Some(ENCODING_SHIFT_JIS); }
    if enc == encoding_rs::EUC_JP { return Some(ENCODING_EUC_JP); }
    if enc == encoding_rs::GBK { return Some(ENCODING_GBK); }
    if enc == encoding_rs::GB18030 { return Some(ENCODING_GB18030); }
    if enc == encoding_rs::BIG5 { return Some(ENCODING_BIG5); }
    if enc == encoding_rs::EUC_KR { return Some(ENCODING_EUC_KR); }
    None
}

pub fn find_charmap(name: &str) -> Option<u8> {
    if let Some(enc) = Encoding::for_label(name.as_bytes()) {
        if let Some(t) = encoding_to_type(enc) {
            return Some(t);
        }
    }
    let bytes = name.as_bytes();
    let mut n = alloc::vec::Vec::new();
    for &c in bytes {
        if c.is_ascii_alphanumeric() {
            n.push(c.to_ascii_lowercase());
        }
    }
    match n.as_slice() {
        b"utf32be" => Some(ENCODING_UTF_32BE),
        b"ucs4be" => Some(ENCODING_UTF_32BE),
        b"utf32le" => Some(ENCODING_UTF_32LE),
        b"ucs4le" => Some(ENCODING_UTF_32LE),
        b"utf32" => Some(ENCODING_UTF_32),
        b"ucs4" => Some(ENCODING_UTF_32),
        b"ucs2be" => Some(ENCODING_UCS2BE),
        b"ucs2le" => Some(ENCODING_UCS2LE),
        b"ucs2" => Some(ENCODING_UCS2),
        b"utf16be" => Some(ENCODING_UTF_16BE),
        b"utf16le" => Some(ENCODING_UTF_16LE),
        b"utf16" => Some(ENCODING_UTF_16),
        b"iso2022jp" => Some(ENCODING_ISO2022_JP),
        b"ascii" => Some(ENCODING_US_ASCII),
        b"usascii" => Some(ENCODING_US_ASCII),
        b"iso646" => Some(ENCODING_US_ASCII),
        b"iso646us" => Some(ENCODING_US_ASCII),
        b"wchart" => Some(ENCODING_WCHAR_T),
        b"gb2312" => Some(ENCODING_GB2312),
        _ => None,
    }
}

pub fn iconv(
    cd: &mut IconvHandle,
    input: &mut &[u8],
    output: &mut &mut [u8],
) -> Result<usize, IconvError> {
    let mut replacements = 0usize;
    if input.is_empty() { return Ok(0); }

    if cd.state == 0 {
        bom_detect(cd, input)?;
    }

    if cd.decoder.is_some() && cd.encoder.is_some() {
        return transcode_both_rs(cd, input, output);
    }

    if cd.decoder.is_some() && cd.encoder.is_none() {
        let decoder = cd.decoder.as_mut().unwrap();
        let mut utf8_buf = vec![0u8; input.len() * 4 + 4];
        let (_, dec_read, dec_written, _) =
            decoder.decode_to_utf8(&input[..], &mut utf8_buf, false);
        *input = &input[dec_read..];
        if dec_written == 0 { return Ok(0); }
        let utf8_str = core::str::from_utf8(&utf8_buf[..dec_written])
            .map_err(|_| IconvError::IllegalSequence)?;
        for ch in utf8_str.chars() {
            if encode_one(cd, ch as u32, output)? {
                replacements += 1;
            }
        }
        return Ok(replacements);
    }

    while !input.is_empty() {
        let (c, consumed) = decode_one(cd, input)?;
        *input = &input[consumed..];
        if encode_one(cd, c, output)? {
            replacements += 1;
        }
    }

    Ok(replacements)
}

fn bom_detect(cd: &mut IconvHandle, input: &mut &[u8]) -> Result<(), IconvError> {
    match cd.from_type {
        ENCODING_UTF_16 | ENCODING_UCS2 => {
            if input.len() < 2 { return Err(IconvError::IncompleteInput); }
            let bom = u16::from_be_bytes([(*input)[0], (*input)[1]]);
            let resolved = if cd.from_type == ENCODING_UCS2 {
                if bom == 0xFFFE { ENCODING_UCS2LE } else { ENCODING_UCS2BE }
            } else {
                if bom == 0xFFFE { ENCODING_UTF_16LE } else { ENCODING_UTF_16BE }
            };
            cd.state = resolved as u32;
            if bom == 0xFFFE || bom == 0xFEFF { *input = &input[2..]; }
            cd.from_type = resolved;
            if cd.decoder.is_none() {
                cd.decoder = resolve_encoding(cd.from_type)
                    .map(|enc| enc.new_decoder_without_bom_handling());
            }
        }
        ENCODING_UTF_32 => {
            if input.len() < 4 { return Err(IconvError::IncompleteInput); }
            let bom = u32::from_be_bytes([(*input)[0], (*input)[1], (*input)[2], (*input)[3]]);
            let resolved = if bom == 0xFFFE0000 { ENCODING_UTF_32LE } else { ENCODING_UTF_32BE };
            cd.state = resolved as u32;
            if bom == 0xFFFE0000 || bom == 0xFEFF { *input = &input[4..]; }
            cd.from_type = resolved;
        }
        _ => {}
    }
    Ok(())
}

fn transcode_both_rs(
    cd: &mut IconvHandle,
    input: &mut &[u8],
    output: &mut &mut [u8],
) -> Result<usize, IconvError> {
    let decoder = cd.decoder.as_mut().unwrap();
    let encoder = cd.encoder.as_mut().unwrap();
    let mut utf8_buf = [0u8; 4];

    while !input.is_empty() {
        let (_, dec_read, dec_written, _) =
            decoder.decode_to_utf8(input, &mut utf8_buf, false);
        if dec_read == 0 && dec_written == 0 { break; }
        *input = &input[dec_read..];

        if dec_written > 0 {
            let src = unsafe { core::str::from_utf8_unchecked(&utf8_buf[..dec_written]) };
            let out = core::mem::take(output);
            let (enc_res, _, enc_written, _) = encoder.encode_from_utf8(src, out, false);
            *output = &mut out[enc_written..];
            if matches!(enc_res, CoderResult::OutputFull) {
                return Err(IconvError::OutputOverflow);
            }
        }
    }
    Ok(0)
}

fn decode_one(cd: &mut IconvHandle, input: &[u8]) -> Result<(u32, usize), IconvError> {
    match cd.from_type {
        ENCODING_US_ASCII => {
            if input.is_empty() { return Err(IconvError::IncompleteInput); }
            let c = input[0] as u32;
            if c >= 128 { return Err(IconvError::IllegalSequence); }
            Ok((c, 1))
        }
        ENCODING_UTF_32BE => {
            if input.len() < 4 { return Err(IconvError::IncompleteInput); }
            let c = u32::from_be_bytes([input[0], input[1], input[2], input[3]]);
            if c.wrapping_sub(0xd800) < 0x800 || c >= 0x110000 {
                return Err(IconvError::IllegalSequence);
            }
            Ok((c, 4))
        }
        ENCODING_UTF_32LE => {
            if input.len() < 4 { return Err(IconvError::IncompleteInput); }
            let c = u32::from_le_bytes([input[0], input[1], input[2], input[3]]);
            if c.wrapping_sub(0xd800) < 0x800 || c >= 0x110000 {
                return Err(IconvError::IllegalSequence);
            }
            Ok((c, 4))
        }
        ENCODING_WCHAR_T => {
            if input.len() < 4 { return Err(IconvError::IncompleteInput); }
            Ok((u32::from_ne_bytes([input[0], input[1], input[2], input[3]]), 4))
        }
        ENCODING_UCS2BE | ENCODING_UTF_16BE => {
            if input.len() < 2 { return Err(IconvError::IncompleteInput); }
            let c = u16::from_be_bytes([input[0], input[1]]) as u32;
            if c.wrapping_sub(0xdc00) < 0x400 { return Err(IconvError::IllegalSequence); }
            if c.wrapping_sub(0xd800) < 0x400 {
                if cd.from_type == ENCODING_UCS2BE { return Err(IconvError::IllegalSequence); }
                if input.len() < 4 { return Err(IconvError::IncompleteInput); }
                let d = u16::from_be_bytes([input[2], input[3]]) as u32;
                if d.wrapping_sub(0xdc00) >= 0x400 { return Err(IconvError::IllegalSequence); }
                return Ok((((c - 0xd7c0) << 10) + (d - 0xdc00), 4));
            }
            Ok((c, 2))
        }
        ENCODING_UCS2LE | ENCODING_UTF_16LE => {
            if input.len() < 2 { return Err(IconvError::IncompleteInput); }
            let c = u16::from_le_bytes([input[0], input[1]]) as u32;
            if c.wrapping_sub(0xdc00) < 0x400 { return Err(IconvError::IllegalSequence); }
            if c.wrapping_sub(0xd800) < 0x400 {
                if cd.from_type == ENCODING_UCS2LE { return Err(IconvError::IllegalSequence); }
                if input.len() < 4 { return Err(IconvError::IncompleteInput); }
                let d = u16::from_le_bytes([input[2], input[3]]) as u32;
                if d.wrapping_sub(0xdc00) >= 0x400 { return Err(IconvError::IllegalSequence); }
                return Ok((((c - 0xd7c0) << 10) + (d - 0xdc00), 4));
            }
            Ok((c, 2))
        }
        _ => Err(IconvError::IllegalSequence),
    }
}

fn encode_one(cd: &mut IconvHandle, c: u32, output: &mut &mut [u8]) -> Result<bool, IconvError> {
    match cd.to_type {
        ENCODING_US_ASCII => {
            if output.is_empty() { return Err(IconvError::OutputOverflow); }
            if c <= 0x7f {
                (**output)[0] = c as u8;
            } else {
                (**output)[0] = b'*';
                let rest = core::mem::take(output);
                *output = &mut rest[1..];
                return Ok(true);
            }
            let rest = core::mem::take(output);
            *output = &mut rest[1..];
            Ok(false)
        }
        ENCODING_UTF_32BE => {
            if output.len() < 4 { return Err(IconvError::OutputOverflow); }
            let bytes = c.to_be_bytes();
            (**output)[..4].copy_from_slice(&bytes);
            let rest = core::mem::take(output);
            *output = &mut rest[4..];
            Ok(false)
        }
        ENCODING_UTF_32LE => {
            if output.len() < 4 { return Err(IconvError::OutputOverflow); }
            let bytes = c.to_le_bytes();
            (**output)[..4].copy_from_slice(&bytes);
            let rest = core::mem::take(output);
            *output = &mut rest[4..];
            Ok(false)
        }
        ENCODING_WCHAR_T => {
            if output.len() < 4 { return Err(IconvError::OutputOverflow); }
            let bytes = c.to_ne_bytes();
            (**output)[..4].copy_from_slice(&bytes);
            let rest = core::mem::take(output);
            *output = &mut rest[4..];
            Ok(false)
        }
        ENCODING_UCS2BE | ENCODING_UCS2LE => {
            if c >= 0x10000 {
                if output.is_empty() { return Err(IconvError::OutputOverflow); }
                (**output)[0] = b'*';
                let rest = core::mem::take(output);
                *output = &mut rest[1..];
                return Ok(true);
            }
            if output.len() < 2 { return Err(IconvError::OutputOverflow); }
            let v = c as u16;
            if cd.to_type == ENCODING_UCS2BE {
                let b = v.to_be_bytes();
                (**output)[0] = b[0]; (**output)[1] = b[1];
            } else {
                let b = v.to_le_bytes();
                (**output)[0] = b[0]; (**output)[1] = b[1];
            }
            let rest = core::mem::take(output);
            *output = &mut rest[2..];
            Ok(false)
        }
        ENCODING_UTF_16BE | ENCODING_UTF_16LE => {
            if c >= 0x10000 {
                if output.len() < 4 { return Err(IconvError::OutputOverflow); }
                let hi = (0xD800 + (((c - 0x10000) >> 10) & 0x3FF)) as u16;
                let lo = (0xDC00 + ((c - 0x10000) & 0x3FF)) as u16;
                if cd.to_type == ENCODING_UTF_16BE {
                    (**output)[0] = (hi >> 8) as u8; (**output)[1] = hi as u8;
                    (**output)[2] = (lo >> 8) as u8; (**output)[3] = lo as u8;
                } else {
                    (**output)[0] = hi as u8; (**output)[1] = (hi >> 8) as u8;
                    (**output)[2] = lo as u8; (**output)[3] = (lo >> 8) as u8;
                }
                let rest = core::mem::take(output);
                *output = &mut rest[4..];
            } else {
                if output.len() < 2 { return Err(IconvError::OutputOverflow); }
                let v = c as u16;
                if cd.to_type == ENCODING_UTF_16BE {
                    (**output)[0] = (v >> 8) as u8; (**output)[1] = v as u8;
                } else {
                    (**output)[0] = v as u8; (**output)[1] = (v >> 8) as u8;
                }
                let rest = core::mem::take(output);
                *output = &mut rest[2..];
            }
            Ok(false)
        }
        _ => encode_one_rs(cd, c, output),
    }
}

fn encode_one_rs(cd: &mut IconvHandle, c: u32, output: &mut &mut [u8]) -> Result<bool, IconvError> {
    let encoder = cd.encoder.as_mut().ok_or(IconvError::IllegalSequence)?;
    let ch = char::from_u32(c).ok_or(IconvError::IllegalSequence)?;
    let mut utf8_buf = [0u8; 4];
    let utf8_str = ch.encode_utf8(&mut utf8_buf);
    let out = core::mem::take(output);
    let (enc_res, _, enc_written, _) = encoder.encode_from_utf8(utf8_str, out, false);
    *output = &mut out[enc_written..];
    if matches!(enc_res, CoderResult::OutputFull) {
        return Err(IconvError::OutputOverflow);
    }
    Ok(false)
}

pub fn iconv_close(cd: IconvHandle) -> Result<(), IconvError> {
    if cd.is_stateful {
        drop(cd);
    }
    Ok(())
}

pub fn iconv_open(to: &str, from: &str) -> Result<IconvHandle, IconvError> {
    let from_type = find_charmap(from).ok_or(IconvError::InvalidEncoding)?;
    let to_type = find_charmap(to).ok_or(IconvError::InvalidEncoding)?;
    let decoder = resolve_encoding(from_type).map(|enc| {
        if from_type == ENCODING_UTF_8 { enc.new_decoder_with_bom_removal() }
        else { enc.new_decoder_without_bom_handling() }
    });
    let encoder = resolve_encoding(to_type).map(|enc| enc.new_encoder());
    let is_stateful = matches!(from_type, ENCODING_UCS2 | ENCODING_UTF_16 | ENCODING_UTF_32 | ENCODING_ISO2022_JP);
    Ok(IconvHandle { from_type, to_type, decoder, encoder, state: 0, is_stateful })
}
