//! Byte offsets (what the Nuclear compiler uses) ↔ LSP positions (line + UTF-16 column).

use nuclear_diag::Span;
use serde_json::{Value, json};

pub struct LineIndex {
    starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(text: &str) -> Self {
        let mut starts = vec![0];
        starts.extend(text.match_indices('\n').map(|(i, _)| i + 1));
        LineIndex { starts }
    }

    /// (line, UTF-16 column), both 0-based.
    pub fn position(&self, text: &str, offset: usize) -> (u32, u32) {
        let offset = clamp(text, offset);
        let line = match self.starts.binary_search(&offset) {
            Ok(i) => i,
            Err(i) => i - 1,
        };
        let col: usize = text[self.starts[line]..offset].chars().map(char::len_utf16).sum();
        (line as u32, col as u32)
    }

    pub fn offset(&self, text: &str, line: u32, character: u32) -> usize {
        let Some(&start) = self.starts.get(line as usize) else { return text.len() };
        let end = self.starts.get(line as usize + 1).copied().unwrap_or(text.len());
        let mut units = 0u32;
        for (i, c) in text[start..end].char_indices() {
            if units >= character || c == '\n' || c == '\r' {
                return start + i;
            }
            units += c.len_utf16() as u32;
        }
        end
    }

    pub fn range(&self, text: &str, span: Span) -> Value {
        let (sl, sc) = self.position(text, span.start);
        let (el, ec) = self.position(text, span.end.max(span.start));
        json!({ "start": { "line": sl, "character": sc }, "end": { "line": el, "character": ec } })
    }

    /// Reads an LSP `Position` object.
    pub fn offset_of(&self, text: &str, pos: &Value) -> usize {
        let line = pos.get("line").and_then(Value::as_u64).unwrap_or(0) as u32;
        let ch = pos.get("character").and_then(Value::as_u64).unwrap_or(0) as u32;
        self.offset(text, line, ch)
    }

    /// Reads an LSP `Range` object.
    pub fn span_of(&self, text: &str, range: &Value) -> Span {
        Span::new(self.offset_of(text, &range["start"]), self.offset_of(text, &range["end"]))
    }
}

/// Moves `offset` back to a char boundary inside `text`.
pub fn clamp(text: &str, offset: usize) -> usize {
    let mut o = offset.min(text.len());
    while !text.is_char_boundary(o) {
        o -= 1;
    }
    o
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16_columns() {
        let text = "a = 'ñ😀'\nb = 1\n";
        let idx = LineIndex::new(text);
        // `'` after the emoji: a = ' ñ 😀 → 5 + 1 + 2 = 8 UTF-16 units
        let quote = text.rfind('\'').unwrap();
        assert_eq!(idx.position(text, quote), (0, 8));
        assert_eq!(idx.offset(text, 0, 8), quote);
        let b = text.find('b').unwrap();
        assert_eq!(idx.position(text, b), (1, 0));
        assert_eq!(idx.offset(text, 1, 0), b);
        assert_eq!(idx.offset(text, 9, 0), text.len());
    }
}
