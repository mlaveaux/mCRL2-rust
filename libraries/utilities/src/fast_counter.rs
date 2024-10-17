use std::fmt;
use std::iter::repeat_with;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use crossbeam_utils::CachePadded;

use crate::thread_id;

/// A sharded atomic counter
///
/// `ConcurrentCounter` shards cacheline aligned `AtomicUsizes` across a vector for faster updates in
/// a high contention scenarios.
pub struct ConcurrentCounter {
    cells: Vec<CachePadded<AtomicUsize>>,
}

impl fmt::Debug for ConcurrentCounter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConcurrentCounter")
            .field("sum", &self.sum())
            .field("shards", &self.cells.len())
            .finish()
    }
}

impl ConcurrentCounter {
    /// Creates a new `ConcurrentCounter` with a minimum of the
    /// `number_of_threads` cells. Concurrent counter will align the
    /// `number_of_threads` to the next power of two for better speed when doing
    /// the modulus.
    #[inline]
    pub fn new(value: usize, number_of_threads: usize) -> Self {
        let number_of_threads = number_of_threads.next_power_of_two();
        let cells: Vec<CachePadded<AtomicUsize>> = repeat_with(|| CachePadded::new(AtomicUsize::new(0)))
            .take(number_of_threads)
            .collect();

        // Make sure the initial value is correct.
        cells[0].store(value, Ordering::Relaxed);

        Self { cells }
    }

    /// Adds `value` to the counter.
    pub fn add(&self, value: usize) {
        let c = self.cells.get(thread_id() & (self.cells.len() - 1)).unwrap();
        c.fetch_add(value, Ordering::Relaxed);
    }

    /// Computes the max of `value` and the counter.
    #[inline]
    pub fn max(&self, value: usize) {
        let c = self.cells.get(thread_id() & (self.cells.len() - 1)).unwrap();
        c.fetch_max(value, Ordering::Relaxed);
    }

    /// This will fetch the sum of the concurrent counter be iterating through
    /// each of the cells and loading the values with the ordering defined by
    /// `ordering`. This is only accurate when all writes have been finished by
    /// the time this function is called.
    #[inline]
    pub fn sum(&self) -> usize {
        self.cells.iter().map(|c| c.load(Ordering::Relaxed)).sum()
    }

    /// This will fetch the max of the concurrent counter be iterating through
    /// each of the cells and loading the values with the ordering defined by
    /// `ordering`. This is only accurate when all writes have been finished by
    /// the time this function is called.
    pub fn total_max(&self) -> usize {
        self.cells.iter().map(|c| c.load(Ordering::Relaxed)).max().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::ConcurrentCounter;

    #[test]
    fn basic_test() {
        let counter = ConcurrentCounter::new(0, 1);
        counter.add(1);
        assert_eq!(counter.sum(), 1);
    }

    #[test]
    fn increment_multiple_times() {
        let counter = ConcurrentCounter::new(0, 1);
        counter.add(1);
        counter.add(1);
        counter.add(1);
        assert_eq!(counter.sum(), 3);
    }

    #[test]
    fn multple_threads_incrementing_multiple_times_concurrently() {
        const WRITE_COUNT: usize = 1_000_000;
        const THREAD_COUNT: usize = 8;

        // Spin up threads that increment the counter concurrently
        let counter = ConcurrentCounter::new(0, THREAD_COUNT as usize);

        std::thread::scope(|s| {
            for _ in 0..THREAD_COUNT {
                s.spawn(|| {
                    for _ in 0..WRITE_COUNT {
                        counter.add(1);
                    }
                });
            }
        });

        assert_eq!(counter.sum(), THREAD_COUNT * WRITE_COUNT);

        assert_eq!(
            format!("Counter is: {counter:?}"),
            "Counter is: ConcurrentCounter { sum: 8000000, shards: 8 }"
        )
    }
}
