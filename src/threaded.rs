use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::thread;
use std::thread::{available_parallelism, JoinHandle};

#[derive(Debug)]
struct Worker<I, O> {
    _handle: JoinHandle<()>,
    input: SyncSender<Option<I>>,
    output: Receiver<Option<O>>,
}

impl<I, O> Worker<I, O>
where
    I: Send + 'static,
    O: Send + 'static,
{
    fn new<F>(func: F) -> Worker<I, O>
    where
        F: Fn(I) -> O + Send + 'static,
    {
        let (input_sender, input_receiver) = sync_channel::<Option<I>>(1);
        let (output_sender, output_receiver) = sync_channel::<Option<O>>(0);

        let handle = thread::spawn(move || {
            for item in input_receiver {
                if output_sender.send(item.map(&func)).is_err() {
                    break;
                }
            }
        });

        Self {
            _handle: handle,
            input: input_sender,
            output: output_receiver,
        }
    }
}

#[derive(Debug)]
pub struct ThreadedIterator<I: Iterator, FI, FO> {
    inner: I,
    workers: Vec<Worker<FI, FO>>,
    input_index: usize,
    num_processing: usize,
}

impl<I, FI, FO> ThreadedIterator<I, FI, FO>
where
    I: Iterator<Item = FI>,
    FI: Send + 'static,
    FO: Send + 'static,
{
    pub fn new<F>(iter: I, func: F) -> ThreadedIterator<I, FI, FO>
    where
        F: Fn(FI) -> FO + Send + Copy + 'static,
    {
        let mut new_iter = Self {
            inner: iter,
            workers: (0..available_parallelism().unwrap().get())
                .map(|_| Worker::new(func))
                .collect(),
            input_index: 0,
            num_processing: 0,
        };

        new_iter.fill_buffer();

        new_iter
    }

    /// Fills the remaining space in the worker queue
    fn fill_buffer(&mut self) {
        while self.num_processing < self.workers.len() {
            let val = self.inner.next();
            self.workers[self.input_index].input.send(val).unwrap();

            // todo: early return instead of flooding threads with None
            self.input_index = (self.input_index + 1) % self.workers.len();
            self.num_processing += 1;
        }
    }

    #[inline]
    fn output_index(&self) -> usize {
        (2 * self.workers.len() + self.input_index - self.num_processing) % self.workers.len()
    }
}

impl<I, FI, FO> Iterator for ThreadedIterator<I, FI, FO>
where
    I: Iterator<Item = FI>,
    FI: Send + 'static,
    FO: Send + 'static,
{
    type Item = FO;

    fn next(&mut self) -> Option<Self::Item> {
        let val = self.workers[self.output_index()].output.recv().unwrap();
        self.num_processing -= 1;

        self.fill_buffer();

        val
    }
}

pub trait IntoThreadedIterator: IntoIterator {
    /// Creates a multithreaded iterator which applies the given function in parallel.
    fn par_map<F, FO>(
        self,
        func: F,
    ) -> ThreadedIterator<Self::IntoIter, <Self as IntoIterator>::Item, FO>
    where
        Self: Sized,
        <Self as IntoIterator>::Item: Send + 'static,
        F: Fn(<Self as IntoIterator>::Item) -> FO + Send + Copy + 'static,
        FO: Send + 'static,
    {
        ThreadedIterator::new(self.into_iter(), func)
    }
}

impl<I: IntoIterator> IntoThreadedIterator for I {}

#[cfg(test)]
mod tests {
    use crate::threaded::{IntoThreadedIterator, Worker};

    #[test]
    fn test_worker() {
        fn square(x: i32) -> i32 {
            x * x
        }

        let pool = Worker::new(&square);

        pool.input.send(Some(10)).unwrap();
        let res = pool.output.recv().unwrap();
        assert_eq!(res, Some(100))
    }

    #[test]
    fn test_iter() {
        fn square(x: i32) -> i32 {
            x * x
        }

        let iter = (0..10).par_map(square).collect::<Vec<_>>();
        assert_eq!(iter, [0, 1, 4, 9, 16, 25, 36, 49, 64, 81])
    }
}
