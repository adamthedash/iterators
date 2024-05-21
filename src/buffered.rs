use std::collections::VecDeque;

#[derive(Debug)]
pub struct BufferedIterator<I: Iterator> {
    inner: I,
    buffer: VecDeque<Option<I::Item>>,
    max_capacity: usize,
}

impl<I: Iterator> BufferedIterator<I> {
    pub fn new(iter: I, capacity: usize) -> BufferedIterator<I> {
        Self {
            inner: iter,
            buffer: VecDeque::with_capacity(capacity),
            max_capacity: capacity,
        }
    }
}

impl<T, I: Iterator<Item=T>> Iterator for BufferedIterator<I>
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.fill_buffer();
        self.buffer.pop_front().unwrap()
    }
}

impl<T, I: Iterator<Item=T>> BufferedIterator<I> {
    fn fill_buffer(&mut self) {
        while self.buffer.len() < self.max_capacity {
            let val = self.inner.next();
            self.buffer.push_back(val);
            if self.buffer[self.buffer.len() - 1].is_none() { return; }
        }
    }
}

pub trait IntoBufferedIterator {
    /// Creates a buffered iterator with the given capacity.
    fn buffered(self, capacity: usize) -> BufferedIterator<Self>
        where
            Self: Sized + Iterator
    {
        BufferedIterator::new(self, capacity)
    }
}

impl<I: Iterator> IntoBufferedIterator for I {}


#[cfg(test)]
mod tests {
    use crate::buffered::IntoBufferedIterator;

    #[test]
    fn test_iter() {
        let iter = (0..10)
            .buffered(10)
            .collect::<Vec<_>>();
        assert_eq!(iter, (0..10).collect::<Vec<_>>())
    }
}