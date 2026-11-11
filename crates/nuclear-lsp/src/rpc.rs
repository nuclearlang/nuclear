//! JSON-RPC 2.0 over stdio with LSP framing (`Content-Length: N\r\n\r\n{json}`).

use serde_json::{Value, json};
use std::io::{self, BufRead, Write};

/// A message from the client: a request (has `id`) or a notification (no `id`).
#[derive(Debug)]
pub struct Incoming {
    pub id: Option<Value>,
    pub method: String,
    pub params: Value,
}

/// Reads one message. `Ok(None)` = end of input. Responses sent by the client are skipped.
pub fn read_message(input: &mut impl BufRead) -> io::Result<Option<Incoming>> {
    loop {
        let mut length: Option<usize> = None;
        let mut line = String::new();
        loop {
            line.clear();
            if input.read_line(&mut line)? == 0 {
                return Ok(None);
            }
            let header = line.trim_end();
            if header.is_empty() {
                if length.is_some() {
                    break;
                }
                continue;
            }
            if let Some((name, value)) = header.split_once(':')
                && name.eq_ignore_ascii_case("content-length")
            {
                length = value.trim().parse().ok();
            }
        }
        let mut body = vec![0; length.unwrap_or(0)];
        input.read_exact(&mut body)?;
        let Ok(msg) = serde_json::from_slice::<Value>(&body) else { continue };
        let Some(method) = msg.get("method").and_then(Value::as_str) else { continue };
        return Ok(Some(Incoming {
            id: msg.get("id").cloned(),
            method: method.to_string(),
            params: msg.get("params").cloned().unwrap_or(Value::Null),
        }));
    }
}

pub struct Sender<W: Write> {
    out: W,
    next_id: i64,
}

impl<W: Write> Sender<W> {
    pub fn new(out: W) -> Self {
        Sender { out, next_id: 0 }
    }

    fn send(&mut self, msg: &Value) {
        let body = msg.to_string();
        let _ = write!(self.out, "Content-Length: {}\r\n\r\n{}", body.len(), body);
        let _ = self.out.flush();
    }

    pub fn respond(&mut self, id: Value, result: Value) {
        self.send(&json!({ "jsonrpc": "2.0", "id": id, "result": result }));
    }

    pub fn respond_error(&mut self, id: Value, code: i64, message: &str) {
        self.send(&json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } }));
    }

    pub fn notify(&mut self, method: &str, params: Value) {
        self.send(&json!({ "jsonrpc": "2.0", "method": method, "params": params }));
    }

    /// A request to the client whose answer is not needed (`workspace/codeLens/refresh`...): the response
    /// is skipped by `read_message`.
    pub fn request(&mut self, method: &str, params: Value) {
        self.next_id += 1;
        let id = format!("nuclear-{}", self.next_id);
        self.send(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
    }
}

pub const METHOD_NOT_FOUND: i64 = -32601;
pub const INVALID_PARAMS: i64 = -32602;
pub const INTERNAL_ERROR: i64 = -32603;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_framed_messages() {
        let a = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let b = r#"{"jsonrpc":"2.0","method":"initialized"}"#;
        let raw =
            format!("Content-Length: {}\r\n\r\n{a}Content-Length: {}\r\nContent-Type: x\r\n\r\n{b}", a.len(), b.len());
        let mut input = io::BufReader::new(raw.as_bytes());
        let first = read_message(&mut input).unwrap().unwrap();
        assert_eq!(first.method, "initialize");
        assert_eq!(first.id, Some(json!(1)));
        let second = read_message(&mut input).unwrap().unwrap();
        assert_eq!(second.method, "initialized");
        assert!(second.id.is_none());
        assert!(read_message(&mut input).unwrap().is_none());
    }
}
