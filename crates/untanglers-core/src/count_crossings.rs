use std::hash::Hash;

use crate::mapping::map_edges;

/**
 * Counts the number of edge crossings in a bipartite graph. This can be done in R * E * ln E time where E is the number of edges.
 * This approach only works if there is at most 1 edge per node-pair. The process works as follows:
 *  1. Sort the edges ascending by their <left node index>, <right node index>
 *  2. Iterate through the sorted edges
 *    a. A new edge crosses every existing edge that has a GREATER right node index (computed using a cumulative sum)
 *    b. The weights are counted multiplicatively (left as an exercise to the reader)
 *    c. Keep track of the number of edges that reach each right node
 *
 * Could likely be further optimised but nowhere near being a bottleneck
 */
pub fn _count_crossings(static_count: usize, mapped_edges: &[(usize, usize, usize)]) -> usize {
  // Step 1
  let mut sorted_edges = mapped_edges.to_owned();
  sorted_edges.sort_unstable();

  let mut weights = vec![0_usize; static_count];
  let mut crossings = 0_usize;

  // Step 2
  for (_, static_id, weight) in sorted_edges {
    crossings += weight * weights[static_id + 1..].iter().sum::<usize>(); // a., b.
    weights[static_id] += weight; // c.
  }

  crossings
}

pub fn count_crossings<T>(nodes1: &[T], nodes2: &[T], edges: &[(T, T, usize)]) -> usize
where
  T: Eq + Hash + Clone,
{
  let mapped_edges = map_edges(nodes1, nodes2, edges);
  _count_crossings(nodes2.len(), &mapped_edges)
}
