use bytes::{Buf, BufMut, BytesMut};
use currentspace_capnweb_core::protocol::Message;
use serde_json;
use std::io;
use tokio_util::codec::{Decoder, Encoder};

/// Codec for Cap'n Web protocol messages
/// Handles serialization/deserialization of protocol messages
pub struct CapnWebCodec {
    /// Maximum frame size to prevent DoS attacks
    max_frame_size: usize,
}

impl CapnWebCodec {
    /// Create a new codec with default settings
    pub fn new() -> Self {
        Self {
            max_frame_size: 10 * 1024 * 1024, // 10MB default
        }
    }

    /// Create a new codec with custom max frame size
    pub fn with_max_frame_size(max_frame_size: usize) -> Self {
        Self { max_frame_size }
    }
}

impl Default for CapnWebCodec {
    fn default() -> Self {
        Self::new()
    }
}

/// Frame format for Cap'n Web messages
#[derive(Debug, Clone)]
pub enum FrameFormat {
    /// Length-prefixed binary frames (4-byte big-endian length)
    LengthPrefixed,
    /// Newline-delimited JSON frames
    NewlineDelimited,
}

/// Decoder for length-prefixed frames
impl Decoder for CapnWebCodec {
    type Item = Message;
    type Error = CodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Need at least 4 bytes for length prefix
        if src.len() < 4 {
            return Ok(None);
        }

        // Read the length prefix (big-endian u32)
        let mut length_bytes = [0u8; 4];
        length_bytes.copy_from_slice(&src[..4]);
        let frame_len = u32::from_be_bytes(length_bytes) as usize;

        // Check frame size limit
        if frame_len > self.max_frame_size {
            return Err(CodecError::FrameTooLarge(frame_len));
        }

        // Check if we have the complete frame
        if src.len() < 4 + frame_len {
            // Need more data
            src.reserve(4 + frame_len - src.len());
            return Ok(None);
        }

        // Extract the frame
        src.advance(4); // Skip length prefix
        let frame_data = src.split_to(frame_len);

        // Parse the JSON message
        let json_value: serde_json::Value = serde_json::from_slice(&frame_data)
            .map_err(|e| CodecError::JsonError(e.to_string()))?;

        // Parse into Cap'n Web message
        let message =
            Message::from_json(&json_value).map_err(|e| CodecError::MessageError(e.to_string()))?;

        Ok(Some(message))
    }
}

/// Encoder for length-prefixed frames
impl Encoder<Message> for CapnWebCodec {
    type Error = CodecError;

    fn encode(&mut self, item: Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Convert message to JSON
        let json_value = item.to_json();
        let json_bytes =
            serde_json::to_vec(&json_value).map_err(|e| CodecError::JsonError(e.to_string()))?;

        // Check frame size
        if json_bytes.len() > self.max_frame_size {
            return Err(CodecError::FrameTooLarge(json_bytes.len()));
        }

        // Write length prefix (big-endian u32)
        let length = json_bytes.len() as u32;
        dst.reserve(4 + json_bytes.len());
        dst.put_u32(length);
        dst.put_slice(&json_bytes);

        Ok(())
    }
}

/// Newline-delimited JSON codec
pub struct NewlineDelimitedCodec {
    max_line_length: usize,
}

impl NewlineDelimitedCodec {
    pub fn new() -> Self {
        Self {
            max_line_length: 1024 * 1024, // 1MB default
        }
    }
}

impl Default for NewlineDelimitedCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder for NewlineDelimitedCodec {
    type Item = Message;
    type Error = CodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Find newline
        let newline_pos = src.iter().position(|&b| b == b'\n');

        if let Some(pos) = newline_pos {
            // Check line length
            if pos > self.max_line_length {
                return Err(CodecError::LineTooLong(pos));
            }

            // Extract the line (without newline)
            let line = src.split_to(pos);
            src.advance(1); // Skip the newline

            // Parse JSON
            let json_value: serde_json::Value =
                serde_json::from_slice(&line).map_err(|e| CodecError::JsonError(e.to_string()))?;

            // Parse message
            let message = Message::from_json(&json_value)
                .map_err(|e| CodecError::MessageError(e.to_string()))?;

            Ok(Some(message))
        } else {
            // Check if buffer is getting too large
            if src.len() > self.max_line_length {
                return Err(CodecError::LineTooLong(src.len()));
            }

            Ok(None)
        }
    }
}

impl Encoder<Message> for NewlineDelimitedCodec {
    type Error = CodecError;

    fn encode(&mut self, item: Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Convert to JSON
        let json_value = item.to_json();
        let json_bytes =
            serde_json::to_vec(&json_value).map_err(|e| CodecError::JsonError(e.to_string()))?;

        // Check line length
        if json_bytes.len() > self.max_line_length {
            return Err(CodecError::LineTooLong(json_bytes.len()));
        }

        // Write JSON and newline
        dst.reserve(json_bytes.len() + 1);
        dst.put_slice(&json_bytes);
        dst.put_u8(b'\n');

        Ok(())
    }
}

/// Codec errors
#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("Frame too large: {0} bytes")]
    FrameTooLarge(usize),

    #[error("Line too long: {0} bytes")]
    LineTooLong(usize),

    #[error("JSON error: {0}")]
    JsonError(String),

    #[error("Message parse error: {0}")]
    MessageError(String),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use currentspace_capnweb_core::{
        protocol::{ExportId, ImportId},
        Expression,
    };

    #[test]
    fn test_length_prefixed_encode_decode() {
        let mut codec = CapnWebCodec::new();
        let mut buffer = BytesMut::new();

        // Encode a message
        let msg = Message::Push(Expression::String("test".to_string()));
        codec.encode(msg.clone(), &mut buffer).unwrap();

        // Check that we have length prefix + data
        assert!(buffer.len() > 4);

        // Decode the message
        let decoded = codec.decode(&mut buffer).unwrap().unwrap();
        match decoded {
            Message::Push(expr) => {
                assert_eq!(expr, Expression::String("test".to_string()));
            }
            _ => panic!("Wrong message type"),
        }

        // Buffer should be empty now
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_newline_delimited_encode_decode() {
        let mut codec = NewlineDelimitedCodec::new();
        let mut buffer = BytesMut::new();

        // Encode a message
        let msg = Message::Pull(ImportId(42));
        codec.encode(msg, &mut buffer).unwrap();

        // Check that it ends with newline
        assert_eq!(buffer[buffer.len() - 1], b'\n');

        // Decode the message
        let decoded = codec.decode(&mut buffer).unwrap().unwrap();
        match decoded {
            Message::Pull(id) => {
                assert_eq!(id, ImportId(42));
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_partial_frame() {
        let mut codec = CapnWebCodec::new();
        let mut buffer = BytesMut::new();

        // Add partial length prefix
        buffer.put_u8(0);
        buffer.put_u8(0);

        // Should return None (need more data)
        assert!(codec.decode(&mut buffer).unwrap().is_none());

        // Add rest of length prefix and partial data
        buffer.put_u8(0);
        buffer.put_u8(10); // Frame length = 10

        // Still need more data
        assert!(codec.decode(&mut buffer).unwrap().is_none());
    }

    #[test]
    fn test_frame_too_large() {
        let mut codec = CapnWebCodec::with_max_frame_size(100);
        let mut buffer = BytesMut::new();

        // Create a message that's too large
        let large_string = "x".repeat(200);
        let msg = Message::Push(Expression::String(large_string));

        // Encoding should fail
        assert!(codec.encode(msg, &mut buffer).is_err());
    }

    #[test]
    fn test_multiple_messages() {
        let mut codec = NewlineDelimitedCodec::new();
        let mut buffer = BytesMut::new();

        // Encode multiple messages
        let msg1 = Message::Push(Expression::String("first".to_string()));
        let msg2 = Message::Pull(ImportId(1));
        let msg3 = Message::Resolve(
            ExportId(-1),
            Expression::Number(serde_json::Number::from(42)),
        );

        codec.encode(msg1, &mut buffer).unwrap();
        codec.encode(msg2, &mut buffer).unwrap();
        codec.encode(msg3, &mut buffer).unwrap();

        // Decode all messages
        let decoded1 = codec.decode(&mut buffer).unwrap().unwrap();
        match decoded1 {
            Message::Push(expr) => {
                assert_eq!(expr, Expression::String("first".to_string()));
            }
            _ => panic!("Wrong message type"),
        }

        let decoded2 = codec.decode(&mut buffer).unwrap().unwrap();
        match decoded2 {
            Message::Pull(id) => {
                assert_eq!(id, ImportId(1));
            }
            _ => panic!("Wrong message type"),
        }

        let decoded3 = codec.decode(&mut buffer).unwrap().unwrap();
        match decoded3 {
            Message::Resolve(id, expr) => {
                assert_eq!(id, ExportId(-1));
                match expr {
                    Expression::Number(n) => assert_eq!(n.as_i64(), Some(42)),
                    _ => panic!("Wrong expression type"),
                }
            }
            _ => panic!("Wrong message type"),
        }
    }
}
