use libc::iconv::{self, IconvError, IconvHandle};

fn conv(
    cd: &mut IconvHandle,
    input: &[u8],
    out_size: usize,
) -> Result<Vec<u8>, IconvError> {
    let mut inp = input;
    let mut out = vec![0u8; out_size];
    let mut out_slice: &mut [u8] = &mut out;
    iconv::iconv(cd, &mut inp, &mut out_slice)?;
    let written = out_size - out_slice.len();
    out.truncate(written);
    Ok(out)
}

#[test]
fn iconv_open_valid() {
    let cd = iconv::iconv_open("UTF-8", "GBK");
    assert!(cd.is_ok());
    let cd = cd.unwrap();
    assert_eq!(cd.from_type, iconv::ENCODING_GBK);
    assert_eq!(cd.to_type, iconv::ENCODING_UTF_8);
}

#[test]
fn iconv_open_invalid_encoding() {
    let cd = iconv::iconv_open("NONEXISTENT-ENC", "UTF-8");
    assert!(matches!(cd, Err(IconvError::InvalidEncoding)));
    let cd = iconv::iconv_open("UTF-8", "NONEXISTENT-ENC");
    assert!(matches!(cd, Err(IconvError::InvalidEncoding)));
}

#[test]
fn iconv_utf8_to_ascii_pure_input() {
    let mut cd = iconv::iconv_open("US-ASCII", "UTF-8").unwrap();
    let out = conv(&mut cd, b"hello", 32).unwrap();
    assert_eq!(&out, b"hello");
}

#[test]
fn iconv_utf8_to_ascii_non_ascii_replaced() {
    let mut cd = iconv::iconv_open("US-ASCII", "UTF-8").unwrap();
    let input = "abc\u{4e2d}".as_bytes();
    let out = conv(&mut cd, input, 32).unwrap();
    assert_eq!(&out, b"abc*");
}

#[test]
fn iconv_empty_input() {
    let mut cd = iconv::iconv_open("UTF-8", "UTF-8").unwrap();
    let out = conv(&mut cd, b"", 32).unwrap();
    assert!(out.is_empty());
}

#[test]
fn iconv_output_overflow() {
    let mut cd = iconv::iconv_open("US-ASCII", "UTF-8").unwrap();
    let result = conv(&mut cd, b"hello world", 4);
    assert!(matches!(result, Err(IconvError::OutputOverflow)));
}

#[test]
fn iconv_gbk_to_utf8() {
    let mut cd = iconv::iconv_open("UTF-8", "GBK").unwrap();
    let gbk_bytes: &[u8] = &[0xd6, 0xd0]; // "中" in GBK
    let out = conv(&mut cd, gbk_bytes, 32).unwrap();
    let s = core::str::from_utf8(&out).unwrap();
    assert_eq!(s, "中");
}

#[test]
fn iconv_utf8_to_gbk() {
    let mut cd = iconv::iconv_open("GBK", "UTF-8").unwrap();
    let input = "中".as_bytes();
    let out = conv(&mut cd, input, 32).unwrap();
    assert_eq!(out.as_slice(), &[0xd6, 0xd0]);
}

#[test]
fn iconv_utf8_to_utf16be() {
    let mut cd = iconv::iconv_open("UTF-16BE", "UTF-8").unwrap();
    let input = "AB".as_bytes();
    let out = conv(&mut cd, input, 32).unwrap();
    assert_eq!(out.as_slice(), &[0x00, 0x41, 0x00, 0x42]);
}

#[test]
fn iconv_utf8_to_utf16le() {
    let mut cd = iconv::iconv_open("UTF-16LE", "UTF-8").unwrap();
    let input = "AB".as_bytes();
    let out = conv(&mut cd, input, 32).unwrap();
    assert_eq!(out.as_slice(), &[0x41, 0x00, 0x42, 0x00]);
}

#[test]
fn iconv_utf8_to_utf32be() {
    let mut cd = iconv::iconv_open("UTF-32BE", "UTF-8").unwrap();
    let input = "A".as_bytes();
    let out = conv(&mut cd, input, 32).unwrap();
    assert_eq!(out.as_slice(), &[0x00, 0x00, 0x00, 0x41]);
}

#[test]
fn iconv_utf8_to_ucs2be() {
    let mut cd = iconv::iconv_open("UCS-2BE", "UTF-8").unwrap();
    let input = "A".as_bytes();
    let out = conv(&mut cd, input, 32).unwrap();
    assert_eq!(out.as_slice(), &[0x00, 0x41]);
}

#[test]
fn iconv_close_basic() {
    let cd = iconv::iconv_open("UTF-8", "GBK").unwrap();
    let result = iconv::iconv_close(cd);
    assert!(result.is_ok());
}

#[test]
fn iconv_find_charmap_aliases() {
    assert_eq!(iconv::find_charmap("UTF-8"), Some(iconv::ENCODING_UTF_8));
    assert_eq!(iconv::find_charmap("utf-8"), Some(iconv::ENCODING_UTF_8));
    assert_eq!(iconv::find_charmap("ASCII"), Some(iconv::ENCODING_US_ASCII));
    assert_eq!(iconv::find_charmap("US-ASCII"), Some(iconv::ENCODING_US_ASCII));
    assert_eq!(iconv::find_charmap("GBK"), Some(iconv::ENCODING_GBK));
    assert_eq!(iconv::find_charmap("GB18030"), Some(iconv::ENCODING_GB18030));
    assert_eq!(iconv::find_charmap("BIG5"), Some(iconv::ENCODING_BIG5));
    assert_eq!(iconv::find_charmap("EUC-KR"), Some(iconv::ENCODING_EUC_KR));
    assert_eq!(iconv::find_charmap("WCHAR_T"), Some(iconv::ENCODING_WCHAR_T));
    assert_eq!(iconv::find_charmap("UCS-2BE"), Some(iconv::ENCODING_UCS2BE));
    assert_eq!(iconv::find_charmap("UTF-16LE"), Some(iconv::ENCODING_UTF_16LE));
    assert_eq!(iconv::find_charmap("UTF-32BE"), Some(iconv::ENCODING_UTF_32BE));
    assert_eq!(iconv::find_charmap("ISO-2022-JP"), Some(iconv::ENCODING_ISO2022_JP));
    assert_eq!(iconv::find_charmap("INVALID"), None);
}

#[test]
fn iconv_roundtrip_gbk() {
    let input = "你好世界hello123".as_bytes();
    let mut cd_enc = iconv::iconv_open("GBK", "UTF-8").unwrap();
    let gbk_out = conv(&mut cd_enc, input, 256).unwrap();
    let mut cd_dec = iconv::iconv_open("UTF-8", "GBK").unwrap();
    let utf8_out = conv(&mut cd_dec, &gbk_out, 256).unwrap();
    assert_eq!(&utf8_out, input);
}

#[test]
fn iconv_roundtrip_big5() {
    let input = "你好測試".as_bytes();
    let mut cd_enc = iconv::iconv_open("BIG5", "UTF-8").unwrap();
    let big5_out = conv(&mut cd_enc, input, 256).unwrap();
    let mut cd_dec = iconv::iconv_open("UTF-8", "BIG5").unwrap();
    let utf8_out = conv(&mut cd_dec, &big5_out, 256).unwrap();
    assert_eq!(&utf8_out, input);
}

#[test]
fn iconv_utf16_bom_detect_be() {
    let mut cd = iconv::iconv_open("UTF-16BE", "UTF-16").unwrap();
    let input: &[u8] = &[0xFE, 0xFF, 0x00, 0x41];
    let out = conv(&mut cd, input, 32).unwrap();
    assert_eq!(out.as_slice(), &[0x00, 0x41]);
}

#[test]
fn iconv_utf16_bom_detect_le() {
    let mut cd = iconv::iconv_open("UTF-16LE", "UTF-16").unwrap();
    let input: &[u8] = &[0xFF, 0xFE, 0x41, 0x00];
    let out = conv(&mut cd, input, 32).unwrap();
    assert_eq!(out.as_slice(), &[0x41, 0x00]);
}
