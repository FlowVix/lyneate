use std::ops::Range;

pub enum SpanOverlay<S> {
    None,
    Single(S),
    Double(S, S),
}

impl<S: MessageSpan> Iterator for SpanOverlay<S> {
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

pub trait MessageSpan: Copy {
    fn start(self) -> usize;
    fn end(self) -> usize;

    fn from_range(r: Range<usize>) -> Self;

    fn to_range(self) -> Range<usize> {
        self.start()..self.end()
    }

    fn len(self) -> usize {
        self.end() - self.start()
    }
    fn is_empty(self) -> bool {
        self.len() == 0
    }

    fn sub(self, n: usize) -> Self {
        Self::from_range((self.start() - n)..(self.end() - n))
    }
    fn plus(self, n: usize) -> Self {
        Self::from_range((self.start() + n)..(self.end() + n))
    }

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
