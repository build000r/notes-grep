use std::io::{self, Read};
use std::str;

use flate2::read::GzDecoder;

const MAX_DECODED_BODY_BYTES: usize = 16 * 1024 * 1024;
const MAX_RECURSION_DEPTH: usize = 8;
const MAX_CANDIDATES: usize = 256;

pub fn decode_note_body(blob: &[u8]) -> std::io::Result<Option<String>> {
    let mut decoded = Vec::new();
    let mut decoder = GzDecoder::new(blob).take((MAX_DECODED_BODY_BYTES + 1) as u64);
    decoder.read_to_end(&mut decoded)?;
    if decoded.len() > MAX_DECODED_BODY_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "decoded note body exceeds size limit",
        ));
    }
    Ok(extract_protobuf_text(&decoded))
}

fn extract_protobuf_text(message: &[u8]) -> Option<String> {
    let mut candidates = Vec::new();
    collect_text_fields(message, 0, &mut candidates);
    candidates.into_iter().max_by_key(|text| text_score(text))
}

fn collect_text_fields(message: &[u8], depth: usize, candidates: &mut Vec<String>) {
    if depth > MAX_RECURSION_DEPTH || candidates.len() >= MAX_CANDIDATES {
        return;
    }

    let mut cursor = 0;
    while cursor < message.len() {
        let Some((key, after_key)) = read_varint(message, cursor) else {
            return;
        };
        if key == 0 {
            return;
        }
        cursor = after_key;

        match key & 0b111 {
            0 => {
                let Some((_, after_value)) = read_varint(message, cursor) else {
                    return;
                };
                cursor = after_value;
            }
            1 => {
                cursor = cursor.saturating_add(8);
            }
            2 => {
                let Some((len, after_len)) = read_varint(message, cursor) else {
                    return;
                };
                let len = len as usize;
                let Some(end) = after_len.checked_add(len) else {
                    return;
                };
                if end > message.len() {
                    return;
                }

                let chunk = &message[after_len..end];
                if let Some(text) = text_candidate(chunk) {
                    candidates.push(text);
                    if candidates.len() >= MAX_CANDIDATES {
                        return;
                    }
                }
                collect_text_fields(chunk, depth + 1, candidates);
                cursor = end;
            }
            5 => {
                cursor = cursor.saturating_add(4);
            }
            _ => return,
        }

        if cursor > message.len() {
            return;
        }
    }
}

fn read_varint(message: &[u8], start: usize) -> Option<(u64, usize)> {
    let mut value = 0u64;
    let mut shift = 0u32;
    let mut cursor = start;

    while cursor < message.len() && shift < 64 {
        let byte = message[cursor];
        cursor += 1;
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Some((value, cursor));
        }
        shift += 7;
    }
    None
}

fn text_candidate(chunk: &[u8]) -> Option<String> {
    let text = str::from_utf8(chunk).ok()?;
    if text.chars().count() < 3 || !text.chars().any(char::is_alphanumeric) {
        return None;
    }

    // The 3-char/alphanumeric guard above guarantees total > 0, so the ratio
    // is well-defined.
    let mut printable = 0usize;
    let mut total = 0usize;
    for ch in text.chars() {
        total += 1;
        if ch == '\n' || ch == '\r' || ch == '\t' || !ch.is_control() {
            printable += 1;
        }
    }
    if printable * 100 / total < 85 {
        return None;
    }

    let normalized = normalize_line_breaks(text);
    let cleaned = normalized
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_owned();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

fn normalize_line_breaks(text: &str) -> String {
    text.chars()
        .map(|ch| match ch {
            '\u{0085}' | '\u{2028}' | '\u{2029}' => '\n',
            _ => ch,
        })
        .collect()
}

fn text_score(text: &str) -> usize {
    let has_newline = usize::from(text.contains('\n')) * 1_000;
    let has_space = usize::from(text.chars().any(char::is_whitespace)) * 500;
    text.len() + has_newline + has_space
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;

    #[test]
    fn decodes_gzipped_protobuf_text_field() {
        let blob = test_body_blob("Fixture body-only phrase lives here");
        let body = decode_note_body(&blob).unwrap().unwrap();
        assert!(body.contains("body-only phrase"));
    }

    #[test]
    fn chooses_nested_long_body_text() {
        let mut nested = Vec::new();
        push_len_field(
            &mut nested,
            2,
            b"Nested body-only phrase with enough text to win scoring",
        );

        let mut outer = Vec::new();
        push_len_field(&mut outer, 1, b"tiny");
        push_len_field(&mut outer, 2, &nested);

        let blob = gzip(&outer);
        let body = decode_note_body(&blob).unwrap().unwrap();
        assert!(body.contains("Nested body-only phrase"));
    }

    #[test]
    fn normalizes_jsonl_hostile_line_separators() {
        let blob = test_body_blob("first\u{2028}second\u{2029}third");
        let body = decode_note_body(&blob).unwrap().unwrap();
        assert_eq!(body, "first\nsecond\nthird");
    }

    #[test]
    fn rejects_oversized_decoded_body() {
        let blob = gzip(&vec![b'a'; MAX_DECODED_BODY_BYTES + 1]);
        let err = decode_note_body(&blob).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn skips_varint_and_fixed_width_fields_before_text() {
        // A realistic Notes protobuf interleaves the text field with non-text
        // scalar fields. Exercise the wire-type skip arms (varint=0, fixed64=1,
        // fixed32=5) and prove the parser still recovers the type-2 text field.
        let mut message = Vec::new();
        // field 1, wire type 0 (varint) = 300; wire type 0 contributes no tag bits
        push_varint(&mut message, 1 << 3);
        push_varint(&mut message, 300);
        // field 2, wire type 1 (fixed64) = 8 bytes
        push_varint(&mut message, (2 << 3) | 1);
        message.extend_from_slice(&[0xAA; 8]);
        // field 3, wire type 5 (fixed32) = 4 bytes
        push_varint(&mut message, (3 << 3) | 5);
        message.extend_from_slice(&[0xBB; 4]);
        // field 4, wire type 2 (length-delimited) = the body text
        push_len_field(&mut message, 4, b"Recovered body text after scalar fields");

        let blob = gzip(&message);
        let body = decode_note_body(&blob).unwrap().unwrap();
        assert_eq!(body, "Recovered body text after scalar fields");
    }

    #[test]
    fn truncated_length_delimited_field_does_not_over_read() {
        // Declare a length-delimited field whose length runs past the buffer.
        // The `end > message.len()` guard must stop parsing without panicking
        // or reading out of bounds, yielding no text candidate.
        let mut message = Vec::new();
        push_varint(&mut message, (1 << 3) | 2); // field 1, wire type 2
        push_varint(&mut message, 1_000); // claims 1000 bytes...
        message.extend_from_slice(b"only a few"); // ...but far fewer follow

        let blob = gzip(&message);
        // Must not panic; no valid text candidate is produced.
        let body = decode_note_body(&blob).unwrap();
        assert_eq!(body, None);
    }

    #[test]
    fn malformed_leading_key_terminates_without_text() {
        // A continuation-only varint (high bit set, never terminating) at the
        // start makes read_varint return None, hitting the early-return arm.
        let message = vec![0x80, 0x80, 0x80];
        let blob = gzip(&message);
        let body = decode_note_body(&blob).unwrap();
        assert_eq!(body, None);
    }

    #[test]
    fn deeply_nested_message_terminates_without_runaway_recursion() {
        // A crafted note body could nest length-delimited messages far deeper
        // than any real note. The depth guard must keep collect_text_fields
        // from recursing arbitrarily deep (which would risk a stack overflow);
        // decoding must still terminate and return a value. We nest well past
        // MAX_RECURSION_DEPTH around a long text leaf.
        let mut payload = Vec::new();
        push_len_field(
            &mut payload,
            1,
            b"Deeply nested body text leaf used to drive recursion depth",
        );
        for _ in 0..(MAX_RECURSION_DEPTH * 4) {
            let mut wrapped = Vec::new();
            push_len_field(&mut wrapped, 1, &payload);
            payload = wrapped;
        }

        let blob = gzip(&payload);
        // The contract under test is termination + no panic on adversarial
        // nesting, not which candidate wins. Decoding must succeed.
        let body = decode_note_body(&blob).unwrap();
        assert!(
            body.is_some(),
            "decoding deeply nested input must terminate"
        );
    }

    fn test_body_blob(text: &str) -> Vec<u8> {
        let mut message = Vec::new();
        push_len_field(&mut message, 1, text.as_bytes());
        gzip(&message)
    }

    fn push_len_field(message: &mut Vec<u8>, field: u64, bytes: &[u8]) {
        push_varint(message, (field << 3) | 2);
        push_varint(message, bytes.len() as u64);
        message.extend_from_slice(bytes);
    }

    fn push_varint(message: &mut Vec<u8>, mut value: u64) {
        while value >= 0x80 {
            message.push((value as u8 & 0x7f) | 0x80);
            value >>= 7;
        }
        message.push(value as u8);
    }

    fn gzip(bytes: &[u8]) -> Vec<u8> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(bytes).unwrap();
        encoder.finish().unwrap()
    }
}
