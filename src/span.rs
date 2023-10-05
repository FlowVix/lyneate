#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct MessageSpan {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl MessageSpan {
    pub(crate) fn size(&self) -> usize {
        self.end - self.start
    }

    pub(crate) fn sub(mut self, n: usize) -> Self {
        self.start -= n;
        self.end -= n;
        self
    }
    pub(crate) fn plus(mut self, n: usize) -> Self {
        self.start += n;
        self.end += n;
        self
    }

    pub(crate) fn overlay(self, over: Self) -> SpanOverlay {
        if over.start == over.end || over.end <= self.start || over.start >= self.end {
            return SpanOverlay::Single(self);
        }
        if over.start <= self.start {
            if over.end >= self.end {
                SpanOverlay::None
            } else {
                SpanOverlay::Single(Self {
                    start: over.end,
                    end: self.end,
                })
            }
        } else {
            #[allow(clippy::collapsible_else_if)]
            if over.end >= self.end {
                SpanOverlay::Single(Self {
                    start: self.start,
                    end: over.start,
                })
            } else {
                SpanOverlay::Double(
                    Self {
                        start: self.start,
                        end: over.start,
                    },
                    Self {
                        start: over.end,
                        end: self.end,
                    },
                )
            }
        }
    }
}

/// The result of overlaying a type implementing [`MessageSpan`] onto another.
/// This result is what remains "visible" of the bottom span.
///
/// ```
/// assert_eq!((10usize..15).overlay(13..17), SpanOverlay::Single(10..13));
/// assert_eq!((10usize..15).overlay(6..12), SpanOverlay::Single(12..15));
/// assert_eq!(
///     (10usize..15).overlay(12..13),
///     SpanOverlay::Double(10..12, 13..15)
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum SpanOverlay {
    None,
    Single(MessageSpan),
    Double(MessageSpan, MessageSpan),
}

impl Iterator for SpanOverlay {
    type Item = MessageSpan;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            SpanOverlay::None => None,
            SpanOverlay::Single(s) => {
                let out = *s;
                *self = SpanOverlay::None;
                Some(out)
            }
            SpanOverlay::Double(s1, s2) => {
                let out = *s1;
                *self = SpanOverlay::Single(*s2);
                Some(out)
            }
        }
    }
}

pub(crate) fn byte_span_to_char_span(text: &str, byte_span: MessageSpan) -> MessageSpan {
    let start = text[..byte_span.start].chars().count();
    let size = text[byte_span.start..byte_span.end].chars().count();
    MessageSpan {
        start,
        end: start + size,
    }
}
