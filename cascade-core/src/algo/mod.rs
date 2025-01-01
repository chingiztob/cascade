pub mod dijkstra;
pub mod itinerary;
pub mod path_wrappers;

pub use itinerary::detailed_itinerary;
pub use path_wrappers::{shortest_path, shortest_path_weight, single_source_shortest_path_weight};

use std::cmp::Ordering;

/// `MinScored<K, T>` holds a score `f64` and a scored object `T` in
/// a pair for use with a `BinaryHeap`.
///
/// `MinScored` compares in reverse order by the score, so that we can
/// use `BinaryHeap` as a min-heap to extract the score-value pair with the
/// least score.
/// This implementation is based on the one in the `petgraph` crate.
#[derive(Copy, Clone, PartialEq)]
pub(crate) struct MinScored<K>(pub f64, pub K);

impl<K: Eq> Eq for MinScored<K> {}

impl<K: PartialOrd> PartialOrd for MinScored<K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.0.partial_cmp(&self.0)
    }
}

impl<K: Ord> Ord for MinScored<K> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.partial_cmp(&self.0).unwrap()
    }
}
