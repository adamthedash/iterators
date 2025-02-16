pub trait Bucket {
    /// Partition the items of this iterator into several buckets based on a bucketing function
    /// The bucketing function must map each item to its associated bucket index.
    /// This function consumes the iterator and stores all results in memory at once, so it's not
    /// suitable for bucketing massive amounts of data or in a streaming fashion.
    fn bucket_arr<F, const N: usize>(self, partition_func: F) -> [Vec<Self::Item>; N]
    where
        Self: Iterator + Sized,
        F: Fn(&Self::Item) -> usize,
    {
        self.fold(std::array::from_fn(|_| vec![]), |mut buckets, item| {
            let index = partition_func(&item);
            assert!(
                index < N,
                "Partition function produced index out of bounds: {}",
                index
            );
            buckets[index].push(item);

            buckets
        })
    }

    /// Partition the items of this iterator into several buckets based on a bucketing function
    /// The bucketing function must map each item to its associated bucket index.
    /// This function consumes the iterator and stores all results in memory at once, so it's not
    /// suitable for bucketing massive amounts of data or in a streaming fashion.
    fn bucket_vec<F>(self, num_buckets: usize, partition_func: F) -> Vec<Vec<Self::Item>>
    where
        Self: Iterator + Sized,
        F: Fn(&Self::Item) -> usize,
    {
        self.fold(
            (0..num_buckets).map(|_| vec![]).collect(),
            |mut buckets, item| {
                let index = partition_func(&item);
                assert!(
                    index < num_buckets,
                    "Partition function produced index out of bounds: {}",
                    index
                );
                buckets[index].push(item);

                buckets
            },
        )
    }

    /// Consumes the iterator, bucketing results into Ok and Err, unwrapping the Result objects
    fn bucket_result<T, E>(self) -> (Vec<T>, Vec<E>)
    where
        Self: Iterator<Item = Result<T, E>> + Sized,
    {
        self.fold((vec![], vec![]), |(mut oks, mut errors), item| {
            match item {
                Ok(t) => oks.push(t),
                Err(e) => errors.push(e),
            };

            (oks, errors)
        })
    }
}

impl<T: Iterator + Sized> Bucket for T {}

#[cfg(test)]
mod tests {
    use super::Bucket;

    #[test]
    fn test_bucket_arr() {
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

        let buckets: [_; 3] = items.iter().bucket_arr(|x| bucket_func(x));
        println!("{:?}", buckets);
        let buckets: [_; 3] = items.into_iter().bucket_arr(bucket_func);
        println!("{:?}", buckets);
    }

    #[test]
    fn test_bucket_vec() {
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

        let buckets = items.iter().bucket_vec(3, |x| bucket_func(x));
        println!("{:?}", buckets);
        let buckets = items.into_iter().bucket_vec(3, bucket_func);
        println!("{:?}", buckets);
    }

    #[test]
    fn test_bucket_result() {
        let items = vec![Ok("One"), Err("Two"), Ok("Three")];

        let (oks, errs) = items.into_iter().bucket_result();
        println!("{:?} {:?}", oks, errs);
    }
}
