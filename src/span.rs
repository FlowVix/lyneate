use std::ops::Range;

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
pub enum SpanOverlay<S> {
    None,
    Single(S),
    Double(S, S),
}

impl<S: MessageSpan + Copy> Iterator for SpanOverlay<S> {
    type Item = S;

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

/// A trait to be implemented for types that represent a byte-aligned or char-aligned
/// span of code.
pub trait MessageSpan: Sized {
    /// The start index of the span.
    fn start(&self) -> usize;
    /// The end index of the span.
    fn end(&self) -> usize;

    /// Creates a span from a [`Range<usize>`]
    fn from_range(r: Range<usize>) -> Self;

    /// Converts the span from a [`Range<usize>`]
    fn to_range(&self) -> Range<usize> {
        self.start()..self.end()
    }

    /// The length of the span
    fn len(&self) -> usize {
        self.end() - self.start()
    }
    /// Returns `true` if the length of the span is `0`
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Shifts the span to the left by `n`
    fn sub(self, n: usize) -> Self {
        Self::from_range((self.start() - n)..(self.end() - n))
    }
    /// Shifts the span to the right by `n`
    fn plus(self, n: usize) -> Self {
        Self::from_range((self.start() + n)..(self.end() + n))
    }

    /// Overlays a type implementing [`MessageSpan`] onto another.
    /// The result is what remains "visible" of the bottom span.
    ///
    /// ```
    /// assert_eq!((10usize..15).overlay(13..17), SpanOverlay::Single(10..13));
    /// assert_eq!((10usize..15).overlay(6..12), SpanOverlay::Single(12..15));
    /// assert_eq!(
    ///     (10usize..15).overlay(12..13),
    ///     SpanOverlay::Double(10..12, 13..15)
    /// );
    /// ```
    fn overlay(self, over: Self) -> SpanOverlay<Self> {
        if over.start() == over.end() || over.end() <= self.start() || over.start() >= self.end() {
            return SpanOverlay::Single(self);
        }
        if over.start() <= self.start() {
            if over.end() >= self.end() {
                SpanOverlay::None
            } else {
                SpanOverlay::Single(Self::from_range(over.end()..self.end()))
            }
        } else {
            #[allow(clippy::collapsible_else_if)]
            if over.end() >= self.end() {
                SpanOverlay::Single(Self::from_range(self.start()..over.start()))
            } else {
                SpanOverlay::Double(
                    Self::from_range(self.start()..over.start()),
                    Self::from_range(over.end()..self.end()),
                )
            }
        }
    }
}

impl MessageSpan for Range<usize> {
    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }

    fn from_range(r: Range<usize>) -> Self {
        r
    }
}
