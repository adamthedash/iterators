# Extended Iterators
Some useful extensions to rust's iterators.  

# Modules
- buffered: Maintains a buffer of items in memory.
- interleave: Interleaves two iterators.  
- logging: Unwraps Result<T> items, debug printing any errors.  
- stateful: Maps a function to items, but allows passing a struct to be used as "working" state. Eg. when the function needs to allocate a lot of memory to compute intermediate values.
- threaded: Multi-threaded map that maintains the ordering of items in the iterator.  
- stateful_threaded: Combination of the stateful and threaded modules.
