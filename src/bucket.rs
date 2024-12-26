pub trait Bucket {
    /// Partition the items of this iterator into several buckets based on a bucketing function
    /// The bucketing function must map each item to its associated bucket index.
    /// This function consumes the iterator and stores all results in memory at once, so it's not
    /// suitable for bucketing massive amounts of data or in a streaming fashion.
    fn bucket<F>(self, num_buckets: usize, partition_func: F) -> Vec<Vec<Self::Item>>
    where
        Self: Iterator + Sized,
        F: Fn(&Self::Item) -> usize,
    {
        let mut buckets = (0..num_buckets).map(|_| vec![]).collect::<Vec<_>>();

        for item in self {
            let index = partition_func(&item);
            buckets[index].push(item);
        }

        buckets
    }
}

impl<T: Iterator + Sized> Bucket for T {}

#[cfg(test)]
mod tests {
    use super::Bucket;

    #[test]
    fn test_bucket() {
        #[derive(Debug)]
        enum TestEnum {
            One,
            Two,
            Three,
        }

        let items = vec![TestEnum::One, TestEnum::Two, TestEnum::Three];
        let bucket_func = |x: &TestEnum| match x {
            TestEnum::One => 0,
            TestEnum::Two => 1,
            TestEnum::Three => 2,
        };

        let buckets = items.iter().bucket(3, |x| bucket_func(x));
        println!("{:?}", buckets);
        let buckets = items.into_iter().bucket(3, bucket_func);
        println!("{:?}", buckets);
    }
}
