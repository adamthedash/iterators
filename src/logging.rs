use std::fmt::Debug;

pub struct LoggingIterator<I: Iterator> {
    inner: I,
}

impl<T, E: Debug, I: Iterator<Item=Result<T, E>>> Iterator for LoggingIterator<I>
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.find_map(|result| match result {
            Ok(val) => Some(val),
            Err(e) => {
                eprintln!("{:?}", e);
                None
            }
        })
    }
}

trait IntoLoggingIterator {
    /// Filters out errors, printing them to stderr. Ok results are unwrapped.
    fn filter_log<T, E: Debug>(self) -> LoggingIterator<Self>
        where
            Self: Sized + Iterator<Item=Result<T, E>>
    {
        LoggingIterator {
            inner: self
        }
    }
}


impl<T, E: Debug, I: Iterator<Item=Result<T, E>>> IntoLoggingIterator for I {}

#[cfg(test)]
mod tests {
    use super::IntoLoggingIterator;

    #[test]
    fn test() {
        let x = [Ok("a"), Err("b"), Ok("c")];
        let y = x.into_iter()
            .filter_log()
            .collect::<Vec<_>>();
        assert_eq!(y, ["a", "c"]);
    }
}