use std::ops::Range;

pub trait MessageSpan: Copy {
    fn start(self) -> usize;
    fn end(self) -> usize;

    fn from_range(r: Range<usize>) -> Self;

    fn to_range(self) -> Range<usize> {
        self.start()..self.end()
    }

    fn sub(self, n: usize) -> Self {
        Self::from_range((self.start() - n)..(self.end() - n))
    }
    fn plus(self, n: usize) -> Self {
        Self::from_range((self.start() + n)..(self.end() + n))
    }
}
