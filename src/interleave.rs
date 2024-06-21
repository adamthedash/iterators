#[derive(Debug)]
struct InterleaveIterator<I, J> {
    left: I,
    right: J,
    next_left: bool,
}

impl<L, R> Iterator for InterleaveIterator<L, R>
    where
        L: Iterator,
        R: Iterator<Item=L::Item>
{
    type Item = L::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let item = match self.next_left {
            true => self.left.next(),
            false => self.right.next(),
        };

        self.next_left = !self.next_left;

        item
    }
}

trait IntoInterleaveIterator: IntoIterator {
    /// Interleaves 2 iterators, starting with the left. Keeps going until one runs out.
    fn interleave<R>(self, other: R) -> InterleaveIterator<Self::IntoIter, R::IntoIter>
        where
            Self: Sized,
            R: IntoIterator<Item=Self::Item>,
    {
        InterleaveIterator {
            left: self.into_iter(),
            right: other.into_iter(),
            next_left: true,
        }
    }
}

impl<I: IntoIterator> IntoInterleaveIterator for I {}

#[cfg(test)]
mod tests {
    use crate::interleave::IntoInterleaveIterator;

    #[test]
    fn test1() {
        let a = vec![1, 2, 3, 4, 5];
        let b = vec![6, 7, 8, 9, 10];
        let out = vec![1, 6, 2, 7, 3, 8, 4, 9, 5, 10];

        // By reference
        let c = a.iter().interleave(&b).collect::<Vec<_>>();
        assert_eq!(out.iter().collect::<Vec<_>>(), c);

        // By value
        let c = a.interleave(b).collect::<Vec<_>>();
        assert_eq!(out, c);
    }

    #[test]
    fn test2() {
        let a = vec![1, 2, 3];
        let b = vec![6, 7, 8, 9, 10];
        let out = vec![1, 6, 2, 7, 3, 8];

        // By reference
        let c = a.iter().interleave(&b).collect::<Vec<_>>();
        assert_eq!(out.iter().collect::<Vec<_>>(), c);

        // By value
        let c = a.interleave(b).collect::<Vec<_>>();
        assert_eq!(out, c);
    }

    #[test]
    fn test3() {
        let a = vec![1, 2, 3, 4, 5];
        let b = vec![6, 7, 8];
        let out = vec![1, 6, 2, 7, 3, 8, 4];

        // By reference
        let c = a.iter().interleave(&b).collect::<Vec<_>>();
        assert_eq!(out.iter().collect::<Vec<_>>(), c);

        // By value
        let c = a.interleave(b).collect::<Vec<_>>();
        assert_eq!(out, c);
    }
}