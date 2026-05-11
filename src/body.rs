use std::io::Read;
use std::str;

use flate2::read::GzDecoder;

const MAX_RECURSION_DEPTH: usize = 8;
const MAX_CANDIDATES: usize = 256;

pub fn decode_note_body(blob: &[u8]) -> std::io::Result<Option<String>> {
    let mut decoded = Vec::new();
    GzDecoder::new(blob).read_to_end(&mut decoded)?;
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
