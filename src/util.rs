use crate::span::MessageSpan;

pub fn byte_span_to_char_span<S: MessageSpan>(text: &str, byte_span: S) -> S {
    let start = text[..byte_span.start()].chars().count();
    let size = text[byte_span.start()..byte_span.end()].chars().count();
    S::from_range(start..start + size)
}
