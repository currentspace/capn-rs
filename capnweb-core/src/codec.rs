use crate::{Message, RpcError};
use bytes::{BufMut, Bytes, BytesMut};
use serde_json;
use std::io::{self, Read, Write};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FrameFormat {
    LengthPrefixed,
    #[default]
    NewlineDelimited,
}

pub fn encode_message(msg: &Message) -> Result<Bytes, RpcError> {
    let json = serde_json::to_vec(msg)?;
    Ok(Bytes::from(json))
}

pub fn decode_message(data: &[u8]) -> Result<Message, RpcError> {
    let msg = serde_json::from_slice(data)?;
    Ok(msg)
}

pub fn encode_frame(msg: &Message, format: FrameFormat) -> Result<Bytes, RpcError> {
    let json = serde_json::to_vec(msg)?;

    match format {
        FrameFormat::LengthPrefixed => {
            let len = json.len() as u32;
            let mut buf = BytesMut::with_capacity(4 + json.len());
            buf.put_u32(len);
            buf.put_slice(&json);
            Ok(buf.freeze())
        }
        FrameFormat::NewlineDelimited => {
            let mut buf = BytesMut::with_capacity(json.len() + 1);
            buf.put_slice(&json);
            buf.put_u8(b'\n');
            Ok(buf.freeze())
        }
    }
}

pub fn decode_frame(data: &[u8], format: FrameFormat) -> Result<(Message, usize), RpcError> {
    match format {
        FrameFormat::LengthPrefixed => {
            if data.len() < 4 {
                return Err(RpcError::bad_request(
                    "Incomplete frame: missing length prefix",
                ));
            }

            let len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
            let total_len = 4 + len;

            if data.len() < total_len {
                return Err(RpcError::bad_request("Incomplete frame: insufficient data"));
            }

            let msg = decode_message(&data[4..total_len])?;
            Ok((msg, total_len))
        }
        FrameFormat::NewlineDelimited => {
            let newline_pos = data
                .iter()
                .position(|&b| b == b'\n')
                .ok_or_else(|| RpcError::bad_request("No newline found in frame"))?;

            let msg = decode_message(&data[..newline_pos])?;
            Ok((msg, newline_pos + 1))
        }
    }
}

pub struct FrameReader<R> {
    reader: R,
    buffer: BytesMut,
    format: FrameFormat,
}

impl<R: Read> FrameReader<R> {
    pub fn new(reader: R, format: FrameFormat) -> Self {
        FrameReader {
            reader,
            buffer: BytesMut::with_capacity(4096),
            format,
        }
    }

    pub fn read_frame(&mut self) -> Result<Option<Message>, RpcError> {
        loop {
            if let Ok((msg, consumed)) = decode_frame(&self.buffer, self.format) {
                self.buffer.advance(consumed);
                return Ok(Some(msg));
            }

            let mut temp_buf = [0u8; 4096];
            match self.reader.read(&mut temp_buf) {
                Ok(0) => return Ok(None),
                Ok(n) => self.buffer.put_slice(&temp_buf[..n]),
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(None),
                Err(e) => return Err(e.into()),
            }
        }
    }
}

pub struct FrameWriter<W> {
    writer: W,
    format: FrameFormat,
}

impl<W: Write> FrameWriter<W> {
    pub fn new(writer: W, format: FrameFormat) -> Self {
        FrameWriter { writer, format }
    }

    pub fn write_frame(&mut self, msg: &Message) -> Result<(), RpcError> {
        let data = encode_frame(msg, self.format)?;
        self.writer.write_all(&data)?;
        self.writer.flush()?;
        Ok(())
    }
}

trait BytesMutExt {
    fn advance(&mut self, cnt: usize);
}

impl BytesMutExt for BytesMut {
    fn advance(&mut self, cnt: usize) {
        if cnt >= self.len() {
            self.clear();
        } else {
            let remaining = self.split_off(cnt);
            *self = remaining;
        }
    }
}

#[cfg(feature = "simd")]
pub mod simd {
    use super::*;
    use simd_json;

    pub fn encode_message_simd(msg: &Message) -> Result<Bytes, RpcError> {
        let json = simd_json::to_vec(msg)
            .map_err(|e| RpcError::bad_request(format!("SIMD JSON encode error: {}", e)))?;
        Ok(Bytes::from(json))
    }

    pub fn decode_message_simd(data: &mut [u8]) -> Result<Message, RpcError> {
        let msg = simd_json::from_slice(data)
            .map_err(|e| RpcError::bad_request(format!("SIMD JSON decode error: {}", e)))?;
        Ok(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{CallId, CapId};
    use crate::msg::{Outcome, Target};
    use serde_json::json;

    #[test]
    fn test_encode_decode_message() {
        let msg = Message::call(
            CallId::new(1),
            Target::cap(CapId::new(42)),
            "test".to_string(),
            vec![json!("hello"), json!(123)],
        );

        let encoded = encode_message(&msg).unwrap();
        let decoded = decode_message(&encoded).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_encode_decode_frame_newline() {
        let msg = Message::cap_ref(CapId::new(99));

        let frame = encode_frame(&msg, FrameFormat::NewlineDelimited).unwrap();
        assert!(frame[frame.len() - 1] == b'\n');

        let (decoded, consumed) = decode_frame(&frame, FrameFormat::NewlineDelimited).unwrap();
        assert_eq!(msg, decoded);
        assert_eq!(consumed, frame.len());
    }

    #[test]
    fn test_encode_decode_frame_length_prefixed() {
        let msg = Message::dispose(vec![CapId::new(1), CapId::new(2), CapId::new(3)]);

        let frame = encode_frame(&msg, FrameFormat::LengthPrefixed).unwrap();
        assert!(frame.len() > 4);

        let (decoded, consumed) = decode_frame(&frame, FrameFormat::LengthPrefixed).unwrap();
        assert_eq!(msg, decoded);
        assert_eq!(consumed, frame.len());
    }

    #[test]
    fn test_frame_reader_writer() {
        use std::io::Cursor;

        let messages = vec![
            Message::call(
                CallId::new(1),
                Target::cap(CapId::new(10)),
                "method".to_string(),
                vec![json!("test")],
            ),
            Message::result(
                CallId::new(1),
                Outcome::Success {
                    value: json!({"result": true}),
                },
            ),
        ];

        let mut buffer = Vec::new();
        {
            let mut writer = FrameWriter::new(&mut buffer, FrameFormat::NewlineDelimited);
            for msg in &messages {
                writer.write_frame(msg).unwrap();
            }
        }

        let cursor = Cursor::new(buffer);
        let mut reader = FrameReader::new(cursor, FrameFormat::NewlineDelimited);

        for expected_msg in messages {
            let msg = reader.read_frame().unwrap().expect("Expected message");
            assert_eq!(msg, expected_msg);
        }

        assert_eq!(reader.read_frame().unwrap(), None);
    }

    #[test]
    fn test_incomplete_frame() {
        let msg = Message::cap_ref(CapId::new(42));
        let frame = encode_frame(&msg, FrameFormat::LengthPrefixed).unwrap();

        let result = decode_frame(&frame[..2], FrameFormat::LengthPrefixed);
        assert!(result.is_err());

        let result = decode_frame(&frame[..frame.len() - 1], FrameFormat::LengthPrefixed);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_frames_in_buffer() {
        let msg1 = Message::cap_ref(CapId::new(1));
        let msg2 = Message::cap_ref(CapId::new(2));

        let frame1 = encode_frame(&msg1, FrameFormat::NewlineDelimited).unwrap();
        let frame2 = encode_frame(&msg2, FrameFormat::NewlineDelimited).unwrap();

        let mut combined = BytesMut::new();
        combined.put_slice(&frame1);
        combined.put_slice(&frame2);

        let (decoded1, consumed1) = decode_frame(&combined, FrameFormat::NewlineDelimited).unwrap();
        assert_eq!(decoded1, msg1);

        let (decoded2, _consumed2) =
            decode_frame(&combined[consumed1..], FrameFormat::NewlineDelimited).unwrap();
        assert_eq!(decoded2, msg2);
    }
}
