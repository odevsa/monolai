/// Percent-decodes a URL-encoded string (e.g. converting %2F -> /, %3A -> :, + -> ' ')
pub fn url_decode(s: &str) -> String {
    let mut bytes = Vec::with_capacity(s.len());
    let input = s.as_bytes();
    let mut i = 0;
    while i < input.len() {
        if input[i] == b'%' && i + 2 < input.len() {
            if let Ok(hex_str) = std::str::from_utf8(&input[i + 1..i + 3]) {
                if let Ok(byte) = u8::from_str_radix(hex_str, 16) {
                    bytes.push(byte);
                    i += 3;
                    continue;
                }
            }
        }
        if input[i] == b'+' {
            bytes.push(b' ');
        } else {
            bytes.push(input[i]);
        }
        i += 1;
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_decode() {
        assert_eq!(
            url_decode("audio-cpp%2Faudio.cpp-gguf%3Akokoro-82m-multilingual-q8_0.gguf%3Aasr"),
            "audio-cpp/audio.cpp-gguf:kokoro-82m-multilingual-q8_0.gguf:asr"
        );
        assert_eq!(url_decode("hello+world"), "hello world");
        assert_eq!(url_decode("hello%20world"), "hello world");
        assert_eq!(url_decode("plain_string"), "plain_string");
        assert_eq!(url_decode("incomplete%"), "incomplete%");
        assert_eq!(url_decode("invalid%ZZ"), "invalid%ZZ");
    }
}

