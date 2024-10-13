pub struct StatefulMapIterator<I, S, F> {
    state: S,
    iter: I,
    func: F,
}

impl<I: Iterator, S, F, FO> Iterator for StatefulMapIterator<I, S, F>
where
    F: FnMut(&mut S, I::Item) -> FO,
{
    type Item = FO;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|x| (self.func)(&mut self.state, x))
    }
}

pub trait IntoStatefulMapIterator: IntoIterator {
    fn stateful_map<S, F>(self, state: S, func: F) -> StatefulMapIterator<Self::IntoIter, S, F>
    where
        Self: Sized,
    {
        StatefulMapIterator {
            iter: self.into_iter(),
            state,
            func,
        }
    }
}

impl<I: IntoIterator> IntoStatefulMapIterator for I {}

#[cfg(test)]
mod tests {
    use crate::stateful::IntoStatefulMapIterator;

    #[test]
    fn test_simple_stateful() {
        struct State {
            total: u8,
        }
        fn cumsum(state: &mut State, x: u8) -> u8 {
            state.total += x;
            state.total
        }

        let values = (0_u8..16).collect::<Vec<_>>();
        let mapped = values
            .stateful_map(State { total: 0 }, cumsum)
            .collect::<Vec<_>>();

        println!("{:?}", mapped);
    }

    #[test]
    fn test_fib() {
        struct State {
            prev0: usize,
            prev1: usize,
        }
        fn fib(state: &mut State, _x: usize) -> usize {
            let next = state.prev0 + state.prev1;
            state.prev0 = state.prev1;
            state.prev1 = next;
            next
        }

        let values = (0..16).collect::<Vec<_>>();
        let mapped = values
            .stateful_map(State { prev0: 1, prev1: 0 }, fib)
            .collect::<Vec<_>>();
        println!("{:?}", mapped);
    }

    #[test]
    fn test_reuse_alloc() {
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

        let values = (0..16).collect::<Vec<_>>();
        let mapped = values
            .stateful_map(
                State {
                    buffer: vec![0; 1000000],
                },
                arbitrary_vector_stuff,
            )
            .collect::<Vec<_>>();
        println!("{:?}", mapped);
    }
}
