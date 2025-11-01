//! Utilities for working with styled text fragments.
//!
//! The types in this module provide a light-weight representation of text "spans" that carry a
//! subset of the styling information supported by [`genpdf`][genpdf].  They are primarily meant to
//! act as an intermediary layer between higher-level rich-text features (such as markdown-like
//! input) and the [`genpdf::elements`] primitives used to render the final PDF document.
//!
//! [genpdf]: https://docs.rs/genpdf/

use std::fmt;

use genpdf::style::{Color, Style, StyledString};

/// A slice of text together with inline style attributes.
///
/// The `Span` type mirrors the most common inline text decorations supported by the PDF renderer
/// (bold, italic and color).  In addition, it exposes an `underline` flag.  The underline effect is
/// not natively supported by `genpdf`'s [`StyledString`], so the conversion helpers in this module
/// keep track of it separately and defer the actual rendering to custom element implementations.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Span {
    text: String,
    bold: bool,
    italic: bool,
    color: Option<Color>,
    underline: bool,
}

impl Span {
    /// Creates a new span with the provided text and no styles applied.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Self::default()
        }
    }

    /// Returns the raw text contained in this span.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns whether the span should be rendered in bold.
    pub fn is_bold(&self) -> bool {
        self.bold
    }

    /// Returns whether the span should be rendered in italic.
    pub fn is_italic(&self) -> bool {
        self.italic
    }

    /// Returns the configured color for the span, if any.
    pub fn color(&self) -> Option<Color> {
        self.color
    }

    /// Returns whether the span is marked as underlined.
    pub fn is_underlined(&self) -> bool {
        self.underline
    }

    /// Sets the bold flag and returns the updated span.
    pub fn with_bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }

    /// Sets the italic flag and returns the updated span.
    pub fn with_italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }

    /// Sets the underline flag and returns the updated span.
    pub fn with_underline(mut self, underline: bool) -> Self {
        self.underline = underline;
        self
    }

    /// Sets the span color and returns the updated span.
    pub fn with_color(mut self, color: Option<Color>) -> Self {
        self.color = color;
        self
    }

    /// Convenience shorthand that marks the span as bold.
    pub fn bold(self) -> Self {
        self.with_bold(true)
    }

    /// Convenience shorthand that marks the span as italic.
    pub fn italic(self) -> Self {
        self.with_italic(true)
    }

    /// Convenience shorthand that marks the span as underlined.
    pub fn underline(self) -> Self {
        self.with_underline(true)
    }

    /// Convenience shorthand that assigns a color to the span.
    pub fn colored(self, color: Color) -> Self {
        self.with_color(Some(color))
    }

    /// Builds a [`Style`] representation for the span.
    fn to_style(&self) -> Style {
        let mut style = Style::new();
        if let Some(color) = self.color {
            style.set_color(color);
        }
        if self.bold {
            style.set_bold();
        }
        if self.italic {
            style.set_italic();
        }
        style
    }

    /// Converts the span to a [`StyledString`] while ignoring the underline attribute.
    ///
    /// The underline information is intentionally dropped at this layer.  Consumers that need to
    /// render underline effects should use [`StyledSpan`] so that the flag is preserved for the
    /// element layer.
    pub fn to_styled_string(&self) -> StyledString {
        StyledString::new(self.text.clone(), self.to_style())
    }
}

impl From<&Span> for StyledString {
    fn from(span: &Span) -> Self {
        span.to_styled_string()
    }
}

impl From<Span> for StyledString {
    fn from(span: Span) -> Self {
        span.to_styled_string()
    }
}

/// A styled span ready to be consumed by `genpdf` elements together with the underline flag.
#[derive(Clone, Debug)]
pub struct StyledSpan {
    /// The styled text fragment.
    pub string: StyledString,
    /// Whether the fragment should be rendered with an underline.
    pub underline: bool,
}

impl StyledSpan {
    /// Creates a new styled span.
    pub fn new(string: StyledString, underline: bool) -> Self {
        Self { string, underline }
    }
}

impl From<&Span> for StyledSpan {
    fn from(span: &Span) -> Self {
        StyledSpan::new(span.to_styled_string(), span.underline)
    }
}

impl From<Span> for StyledSpan {
    fn from(span: Span) -> Self {
        StyledSpan::from(&span)
    }
}

/// Converts a sequence of [`Span`] values into styled strings while keeping underline flags.
pub fn spans_to_styled_strings<'a, I>(spans: I) -> Vec<StyledSpan>
where
    I: IntoIterator<Item = &'a Span>,
{
    spans.into_iter().map(StyledSpan::from).collect()
}

/// Parse errors produced by [`parse_markup`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    index: usize,
    message: String,
}

impl ParseError {
    fn new(index: usize, message: impl Into<String>) -> Self {
        Self {
            index,
            message: message.into(),
        }
    }

    /// Byte index in the original input string where the error was detected.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Human-readable description of the parsing error.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (at byte {})", self.message, self.index)
    }
}

impl std::error::Error for ParseError {}

#[derive(Clone, Copy, Debug, Default)]
struct StyleState {
    bold: bool,
    italic: bool,
    color: Option<Color>,
    underline: bool,
}

impl StyleState {
    fn to_span(&self, text: impl Into<String>) -> Span {
        Span {
            text: text.into(),
            bold: self.bold,
            italic: self.italic,
            color: self.color,
            underline: self.underline,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Marker {
    Bold,
    Italic,
    Color,
}

impl Marker {
    fn closing_token(self) -> &'static str {
        match self {
            Marker::Bold => "**",
            Marker::Italic => "*",
            Marker::Color => "}",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Marker::Bold => "bold span",
            Marker::Italic => "italic span",
            Marker::Color => "color span",
        }
    }
}

/// Parses a small markdown-inspired syntax into a list of [`Span`]s.
///
/// The supported constructs are:
///
/// - `**bold**` for bold text
/// - `*italic*` for italic text
/// - `[color=#RRGGBB]{text}` for colored text, where `RRGGBB` is a hexadecimal RGB value
///
/// The parser performs strict validation and returns [`ParseError`] with positional information for
/// malformed inputs.  The underline flag is not exposed through this syntax, but callers may set it
/// on the returned spans if required.
pub fn parse_markup(input: &str) -> Result<Vec<Span>, ParseError> {
    let (spans, idx) = parse_inner(input, 0, StyleState::default(), None)?;
    debug_assert_eq!(idx, input.len());
    Ok(spans)
}

fn parse_inner(
    input: &str,
    mut index: usize,
    state: StyleState,
    closing_marker: Option<Marker>,
) -> Result<(Vec<Span>, usize), ParseError> {
    let mut spans = Vec::new();
    let mut buffer = String::new();

    while index < input.len() {
        if let Some(marker) = closing_marker {
            if input[index..].starts_with(marker.closing_token()) {
                flush_buffer(&mut buffer, &mut spans, state);
                index += marker.closing_token().len();
                return Ok((spans, index));
            }
        }

        if input[index..].starts_with("**") {
            flush_buffer(&mut buffer, &mut spans, state);
            index += 2;
            let mut nested_state = state;
            nested_state.bold = true;
            let (nested, new_index) = parse_inner(input, index, nested_state, Some(Marker::Bold))?;
            spans.extend(nested);
            index = new_index;
            continue;
        }

        if input[index..].starts_with('*') {
            flush_buffer(&mut buffer, &mut spans, state);
            index += 1;
            let mut nested_state = state;
            nested_state.italic = true;
            let (nested, new_index) =
                parse_inner(input, index, nested_state, Some(Marker::Italic))?;
            spans.extend(nested);
            index = new_index;
            continue;
        }

        if input[index..].starts_with("[color=") {
            let (color, after_directive) = parse_color_directive(input, index)?;
            flush_buffer(&mut buffer, &mut spans, state);
            let mut nested_state = state;
            nested_state.color = Some(color);
            index = after_directive;
            let (nested, new_index) = parse_inner(input, index, nested_state, Some(Marker::Color))?;
            spans.extend(nested);
            index = new_index;
            continue;
        }

        if input[index..].starts_with('}') {
            return Err(ParseError::new(
                index,
                "unexpected closing token `}` without matching opening `[color=...]`",
            ));
        }

        if input[index..].starts_with(']') {
            return Err(ParseError::new(index, "unexpected closing token `]`"));
        }

        if input[index..].starts_with('[') {
            return Err(ParseError::new(
                index,
                "unsupported directive; expected `[color=#RRGGBB]{...}`",
            ));
        }

        let ch = input[index..]
            .chars()
            .next()
            .expect("character extraction succeeded");
        buffer.push(ch);
        index += ch.len_utf8();
    }

    if let Some(marker) = closing_marker {
        Err(ParseError::new(
            index,
            format!("unterminated {}", marker.description()),
        ))
    } else {
        flush_buffer(&mut buffer, &mut spans, state);
        Ok((spans, index))
    }
}

fn flush_buffer(buffer: &mut String, spans: &mut Vec<Span>, state: StyleState) {
    if buffer.is_empty() {
        return;
    }
    spans.push(state.to_span(std::mem::take(buffer)));
}

fn parse_color_directive(input: &str, index: usize) -> Result<(Color, usize), ParseError> {
    const PREFIX: &str = "[color=";
    let start_hex = index + PREFIX.len();
    if !input[start_hex..].starts_with('#') {
        return Err(ParseError::new(
            start_hex,
            "expected `#` followed by a hexadecimal RGB value",
        ));
    }

    let hex_start = start_hex + 1;
    let hex_end = hex_start + 6;
    if hex_end > input.len() {
        return Err(ParseError::new(
            hex_start,
            "incomplete color specification; expected 6 hexadecimal digits",
        ));
    }

    let hex = &input[hex_start..hex_end];
    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ParseError::new(
            hex_start,
            "invalid RGB specification; use hexadecimal digits only",
        ));
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap();

    let bracket_index = hex_end;
    if !input[bracket_index..].starts_with(']') {
        return Err(ParseError::new(
            bracket_index,
            "expected `]` to close color directive",
        ));
    }

    let brace_index = bracket_index + 1;
    if brace_index >= input.len() || !input[brace_index..].starts_with('{') {
        return Err(ParseError::new(
            brace_index,
            "expected `{` to start the colored text",
        ));
    }

    Ok((Color::Rgb(r, g, b), brace_index + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_to_style_reflects_flags() {
        let span = Span::new("Hello")
            .bold()
            .italic()
            .colored(Color::Rgb(10, 20, 30));
        let styled = span.to_styled_string();
        assert_eq!(styled.s, "Hello");
        assert!(styled.style.is_bold());
        assert!(styled.style.is_italic());
        assert_eq!(styled.style.color(), Some(Color::Rgb(10, 20, 30)));
    }

    #[test]
    fn styled_span_captures_underline_flag() {
        let span = Span::new("Underline me").underline();
        let styled = StyledSpan::from(&span);
        assert_eq!(styled.string.s, "Underline me");
        assert!(styled.underline);
    }

    #[test]
    fn parse_plain_text() {
        let spans = parse_markup("Hello world").expect("parse succeeds");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].text(), "Hello world");
        assert!(!spans[0].is_bold());
    }

    #[test]
    fn parse_nested_styles() {
        let spans = parse_markup("This is **very *cool***!").expect("parse succeeds");
        assert_eq!(spans.len(), 4);
        assert_eq!(spans[0].text(), "This is ");
        assert!(!spans[0].is_bold());
        assert!(spans[1].is_bold());
        assert_eq!(spans[1].text(), "very ");
        assert!(spans[2].is_bold());
        assert!(spans[2].is_italic());
        assert_eq!(spans[2].text(), "cool");
        assert_eq!(spans[3].text(), "!");
        assert!(!spans[3].is_bold());
    }

    #[test]
    fn parse_color_directive() {
        let spans = parse_markup("[color=#ff0000]{Red} text").expect("parse succeeds");
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].text(), "Red");
        assert_eq!(spans[0].color(), Some(Color::Rgb(0xff, 0x00, 0x00)));
        assert_eq!(spans[1].text(), " text");
    }

    #[test]
    fn error_on_unterminated_bold() {
        let err = parse_markup("**oops").unwrap_err();
        assert!(err.message().contains("unterminated bold"));
    }

    #[test]
    fn error_on_invalid_color() {
        let err = parse_markup("[color=#12FG34]{x}").unwrap_err();
        assert!(err.message().contains("invalid RGB"));
    }
}
