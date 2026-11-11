//! Doc comments: the `//` comment right above a declaration, with JSDoc-style tags (base §2.3).
//!
//! ```text
//! // Price with tax, rounded to cents.
//!   @param amount price before tax
//!   @return the price with 16% tax
//!   @example
//!     withTax(100.0)   // 116.0
//! withTax(amount: double) double
//! ```
//!
//! A line that starts with `@word` opens a tag; the lines after it continue it. The lines of `@example`
//! that are more indented than the tag are code and keep their layout.

/// The parts of a doc comment. Unknown tags go to `other`.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DocComment {
    /// Description lines; an empty line separates paragraphs.
    pub summary: Vec<String>,
    /// `@param name text`
    pub params: Vec<(String, String)>,
    /// `@return text` (or `@returns`)
    pub returns: Option<String>,
    /// `@throws text`
    pub throws: Vec<String>,
    /// `@example [caption]` + more indented lines: (caption, code).
    pub examples: Vec<(String, String)>,
    /// `@see text`
    pub see: Vec<String>,
    /// `@deprecated [reason]`: `Some("")` without a reason.
    pub deprecated: Option<String>,
    /// `@since version`
    pub since: Option<String>,
    /// Any other `@tag text`.
    pub other: Vec<(String, String)>,
}

enum Open {
    Summary,
    Param,
    Returns,
    Throws,
    See,
    Deprecated,
    Since,
    Other,
    /// Indentation of the `@example` line: more indented lines are its code.
    Example(usize),
}

fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

/// `@param` → `param`, for a line that starts with a tag.
fn tag_of(line: &str) -> Option<(&str, &str)> {
    let rest = line.trim_start().strip_prefix('@')?;
    let end = rest.find(|c: char| !c.is_ascii_alphanumeric()).unwrap_or(rest.len());
    let (tag, text) = rest.split_at(end);
    (!tag.is_empty() && (text.is_empty() || text.starts_with(char::is_whitespace))).then_some((tag, text.trim()))
}

fn push_text(slot: &mut String, line: &str) {
    if !slot.is_empty() {
        slot.push(' ');
    }
    slot.push_str(line);
}

/// Removes the indentation all non-empty lines share.
fn dedent(lines: &[&str]) -> String {
    let min = lines.iter().filter(|l| !l.trim().is_empty()).map(|l| indent_of(l)).min().unwrap_or(0);
    let out: Vec<&str> = lines.iter().map(|l| if l.len() >= min { &l[min..] } else { l.trim_start() }).collect();
    out.join("\n").trim_end().to_string()
}

impl DocComment {
    /// Reads the text of a comment (without `//`; lines separated by `\n`).
    pub fn parse(text: &str) -> DocComment {
        let mut doc = DocComment::default();
        let mut open = Open::Summary;
        let mut code: Vec<&str> = Vec::new();
        for line in text.lines() {
            if let Open::Example(at) = open {
                if line.trim().is_empty() || indent_of(line) > at {
                    code.push(line);
                    continue;
                }
                doc.examples.last_mut().expect("example opened").1 = dedent(&code);
                code.clear();
                open = Open::Summary;
            }
            if let Some((tag, rest)) = tag_of(line) {
                open = match tag {
                    "param" => {
                        let (name, text) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
                        doc.params.push((name.trim_end_matches(':').to_string(), text.trim().to_string()));
                        Open::Param
                    }
                    "return" | "returns" => {
                        doc.returns = Some(rest.to_string());
                        Open::Returns
                    }
                    "throws" | "throw" => {
                        doc.throws.push(rest.to_string());
                        Open::Throws
                    }
                    "example" => {
                        doc.examples.push((rest.to_string(), String::new()));
                        Open::Example(indent_of(line))
                    }
                    "see" => {
                        doc.see.push(rest.to_string());
                        Open::See
                    }
                    "deprecated" => {
                        doc.deprecated = Some(rest.to_string());
                        Open::Deprecated
                    }
                    "since" => {
                        doc.since = Some(rest.to_string());
                        Open::Since
                    }
                    _ => {
                        doc.other.push((tag.to_string(), rest.to_string()));
                        Open::Other
                    }
                };
                continue;
            }
            let line = line.trim();
            match open {
                Open::Summary | Open::Example(_) => doc.summary.push(line.to_string()),
                Open::Param => push_text(&mut doc.params.last_mut().expect("param").1, line),
                Open::Returns => push_text(doc.returns.get_or_insert_default(), line),
                Open::Throws => push_text(doc.throws.last_mut().expect("throws"), line),
                Open::See => push_text(doc.see.last_mut().expect("see"), line),
                Open::Deprecated => push_text(doc.deprecated.get_or_insert_default(), line),
                Open::Since => push_text(doc.since.get_or_insert_default(), line),
                Open::Other => push_text(&mut doc.other.last_mut().expect("tag").1, line),
            }
        }
        if let Some(e) = doc.examples.last_mut().filter(|_| !code.is_empty()) {
            e.1 = dedent(&code);
        }
        while doc.summary.last().is_some_and(String::is_empty) {
            doc.summary.pop();
        }
        doc
    }

    /// Has any tag (more than a description).
    pub fn has_tags(&self) -> bool {
        !(self.params.is_empty()
            && self.returns.is_none()
            && self.throws.is_empty()
            && self.examples.is_empty()
            && self.see.is_empty()
            && self.deprecated.is_none()
            && self.since.is_none()
            && self.other.is_empty())
    }

    /// Text of `@param name`.
    pub fn param(&self, name: &str) -> Option<&str> {
        self.params.iter().find(|(n, _)| n == name).map(|(_, t)| t.as_str())
    }

    /// Markdown for an editor: description, then a section per tag. Examples are Nuclear code.
    pub fn markdown(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if let Some(why) = &self.deprecated {
            parts.push(if why.is_empty() { "**Deprecated.**".into() } else { format!("**Deprecated:** {why}") });
        }
        if !self.summary.is_empty() {
            parts.push(self.summary.join("\n"));
        }
        if !self.params.is_empty() {
            let list: Vec<String> = self
                .params
                .iter()
                .map(|(n, t)| if t.is_empty() { format!("- `{n}`") } else { format!("- `{n}` — {t}") })
                .collect();
            parts.push(format!("**Parameters**\n{}", list.join("\n")));
        }
        if let Some(r) = &self.returns {
            parts.push(format!("**Returns** {r}"));
        }
        for t in &self.throws {
            parts.push(format!("**Throws** {t}"));
        }
        for (caption, code) in &self.examples {
            let title =
                if caption.is_empty() { "**Example**".to_string() } else { format!("**Example** — {caption}") };
            parts.push(format!("{title}\n```nuclear\n{code}\n```"));
        }
        for s in &self.see {
            parts.push(format!("**See** {s}"));
        }
        if let Some(v) = &self.since {
            parts.push(format!("*Since {v}*"));
        }
        for (tag, t) in &self.other {
            parts.push(format!("**@{tag}** {t}"));
        }
        parts.join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_tags() {
        let d = DocComment::parse(
            "Price with tax.\nRounded to cents.\n@param amount price\n  before tax\n@return the price\n@throws RangeError if negative\n@example simple use\n  x = withTax(100.0)\n    @Ignored(x)\n@deprecated use total()\n@see total\n@since 0.2\n@todo later",
        );
        assert_eq!(d.summary, ["Price with tax.", "Rounded to cents."]);
        assert_eq!(d.params, [("amount".to_string(), "price before tax".to_string())]);
        assert_eq!(d.returns.as_deref(), Some("the price"));
        assert_eq!(d.throws, ["RangeError if negative"]);
        assert_eq!(d.examples, [("simple use".to_string(), "x = withTax(100.0)\n  @Ignored(x)".to_string())]);
        assert_eq!(d.deprecated.as_deref(), Some("use total()"));
        assert_eq!(d.see, ["total"]);
        assert_eq!(d.since.as_deref(), Some("0.2"));
        assert_eq!(d.other, [("todo".to_string(), "later".to_string())]);
        assert_eq!(d.param("amount"), Some("price before tax"));
        let md = d.markdown();
        assert!(md.starts_with("**Deprecated:** use total()\n\nPrice with tax."), "{md}");
        assert!(md.contains("- `amount` — price before tax"), "{md}");
        assert!(md.contains("```nuclear\nx = withTax(100.0)\n  @Ignored(x)\n```"), "{md}");
    }

    #[test]
    fn plain_comment_is_only_a_summary() {
        let d = DocComment::parse("Just text\nmore text, with an email a@b.c");
        assert_eq!(d.summary.len(), 2);
        assert!(!d.has_tags());
        assert_eq!(d.markdown(), "Just text\nmore text, with an email a@b.c");
    }

    #[test]
    fn example_ends_at_a_less_indented_line() {
        let d = DocComment::parse("@example\n  f(1)\n\n  f(2)\n@return x");
        assert_eq!(d.examples[0].1, "f(1)\n\nf(2)");
        assert_eq!(d.returns.as_deref(), Some("x"));
    }
}
