use std::sync::mpsc::sync_channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::thread;
use std::thread::available_parallelism;
use std::thread::JoinHandle;

#[derive(Debug)]
struct StatefulWorker<I, O> {
    _handle: JoinHandle<()>,
    input: SyncSender<Option<I>>,
    output: Receiver<Option<O>>,
}

impl<I, O> StatefulWorker<I, O>
where
    I: Send + 'static,
    O: Send + 'static,
{
    fn new<F, S>(func: F, mut state: S) -> StatefulWorker<I, O>
    where
        F: FnMut(&mut S, I) -> O + Send + 'static,
        S: Send + 'static,
    {
        let (input_sender, input_receiver) = sync_channel::<Option<I>>(1);
        let (output_sender, output_receiver) = sync_channel::<Option<O>>(0);

        let handle = thread::spawn(move || {
            let mut func = func; // Save func as mutable here. Because we are passing a mutable
                                 // state to the closure, the function must be mutable aswell.
            for item in input_receiver {
                let output_item = item.map(|x| func(&mut state, x));
                if output_sender.send(output_item).is_err() {
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
pub struct ThreadedStatefulIterator<I: Iterator, FI, FO> {
    inner: I,
    workers: Vec<StatefulWorker<FI, FO>>,
    input_index: usize,
    num_processing: usize,
}

impl<I, FI, FO> ThreadedStatefulIterator<I, FI, FO>
where
    I: Iterator<Item = FI>,
    FI: Send + 'static,
    FO: Send + 'static,
{
    pub fn new<F, S>(iter: I, func: F, state: S) -> ThreadedStatefulIterator<I, FI, FO>
    where
        F: FnMut(&mut S, FI) -> FO + Send + Copy + 'static,
        S: Send + Clone + 'static,
    {
        let mut workers = vec![];
        for _ in 0..available_parallelism().unwrap().get() {
            workers.push(StatefulWorker::new(func, state.clone()));
        }

        let mut new_iter = Self {
            inner: iter,
            workers,
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

impl<I, FI, FO> Iterator for ThreadedStatefulIterator<I, FI, FO>
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

pub trait IntoStatefulThreadedIterator: IntoIterator {
    /// Creates a multithreaded iterator which applies the given function in parallel.
    /// Note: State here is meant to be used as working memory for each thread, don't rely on it to
    /// accumulate values/pass information between function executions.
    fn stateful_par_map<F, FO, S>(
        self,
        func: F,
        state: S,
    ) -> ThreadedStatefulIterator<Self::IntoIter, <Self as IntoIterator>::Item, FO>
    where
        Self: Sized,
        <Self as IntoIterator>::Item: Send + 'static,
        F: Fn(&mut S, <Self as IntoIterator>::Item) -> FO + Send + Copy + 'static,
        FO: Send + 'static,
        S: Send + Clone + 'static,
    {
        ThreadedStatefulIterator::new(self.into_iter(), func, state)
    }
}

impl<I: IntoIterator> IntoStatefulThreadedIterator for I {}

#[cfg(test)]
mod tests {

    use super::StatefulWorker;
    use crate::stateful_threaded::IntoStatefulThreadedIterator;

    #[test]
    fn test_worker() {
        struct State {
            total: u8,
        }
        fn cumsum(state: &mut State, x: u8) -> u8 {
            state.total += x;
            state.total
        }

        let pool = StatefulWorker::new(&cumsum, State { total: 0 });
        let values = (0..16).collect::<Vec<_>>();
        for x in values {
            pool.input.send(Some(x)).unwrap();
            let res = pool.output.recv().unwrap();
            println!("{:?}", res);
        }
    }

    #[test]
    fn test_iterator() {
        #[derive(Clone)]
        struct State {
            total: usize,
        }
        fn cumsum(state: &mut State, x: usize) -> usize {
            state.total += x;
            state.total
        }

        let values = (0..128).collect::<Vec<_>>();

        let mapped = values
            .stateful_par_map(cumsum, State { total: 0 })
            .collect::<Vec<_>>();
        println!("{:?}", mapped);
    }

    #[test]
    fn test_iterator_array() {
        #[derive(Clone)]
        struct State {
            buffer: Vec<u8>,
        }
        fn arbitrary_vector_stuff(state: &mut State, _x: u8) -> u8 {
            // Simulate some expensive operations on an array
            for item in state.buffer.iter_mut() {
                for _ in 0..100 {
                    *item += 1;
                    *item -= 1;
                }
            }
            _x
        }

        let values = (0..128).collect::<Vec<_>>();
        let mapped = values
            .stateful_par_map(
                arbitrary_vector_stuff,
                State {
                    buffer: vec![0; 1000000],
                },
            )
            .collect::<Vec<_>>();
    }
}
