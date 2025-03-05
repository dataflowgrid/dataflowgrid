#[derive(Debug)]
pub enum TextDecoderError {
    BytesLeftAfterFinish,
}

pub struct TextDecoderResult {
    pub consumed_bytes: usize,
    pub generated_chars: usize,
}

pub trait TextDecoder {
    fn decode(&self, bytes: &[u8], chars: &mut [char]) -> Result<TextDecoderResult, TextDecoderError>;
    fn finish(&self, _chars: &mut [char]) -> Result<TextDecoderResult, TextDecoderError> {
        Ok(
            TextDecoderResult {
                consumed_bytes: 0,
                generated_chars: 0,
            })
    }
}

// Implementations

pub struct UTF8Decoder {
}

impl TextDecoder for UTF8Decoder {
    fn decode(&self, bytes: &[u8], chars: &mut [char]) -> Result<TextDecoderResult, TextDecoderError> {
        Ok(
            TextDecoderResult {
                consumed_bytes: 0,
                generated_chars: 0,
            })
    }
}

pub struct UTF16LEDecoder {
}

impl TextDecoder for UTF16LEDecoder {
    fn decode(&self, bytes: &[u8], chars: &mut [char]) -> Result<TextDecoderResult, TextDecoderError> {
        Ok(
            TextDecoderResult {
                consumed_bytes: 0,
                generated_chars: 0,
            })
    }
}

pub struct UTF16BEDecoder {
}

impl TextDecoder for UTF16BEDecoder {
    fn decode(&self, bytes: &[u8], chars: &mut [char]) -> Result<TextDecoderResult, TextDecoderError> {
        Ok(
            TextDecoderResult {
                consumed_bytes: 0,
                generated_chars: 0,
            })
    }
}

pub struct ASCIIDecoder {
}

impl ASCIIDecoder {
    pub fn new() -> ASCIIDecoder {
        ASCIIDecoder {}
    }
}

impl TextDecoder for ASCIIDecoder {
    
    fn decode(&self, bytes: &[u8], chars: &mut [char]) -> Result<TextDecoderResult, TextDecoderError> {
        let mut i = 0;
        while i < bytes.len() && i < chars.len() {
            let byte = &bytes[i];
            chars[i] = *byte as char;
            i += 1;
        }
        Ok(
            TextDecoderResult {
                consumed_bytes: i,
                generated_chars: i,
            })
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn ascii_hello_world() {
        let decoder = ASCIIDecoder::new();
        let bytes = "Hello, World!".as_bytes();
        let mut chars = ['\0'; 13];
        let result = decoder.decode(bytes, &mut chars);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.consumed_bytes, 13);
        assert_eq!(result.generated_chars, 13);
        assert_eq!(chars, ['H', 'e', 'l', 'l', 'o', ',', ' ', 'W', 'o', 'r', 'l', 'd', '!']);

        let result = decoder.finish(&mut chars).unwrap();
        assert_eq!(result.consumed_bytes, 0);
        assert_eq!(result.generated_chars, 0);
    }
}
