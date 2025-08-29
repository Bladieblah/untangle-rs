use itertools::Itertools;
use rand::random;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;

use crate::utils::matmul;

fn invert_vec<T>(v: &[T]) -> HashMap<&T, usize>
where
  T: Eq + Hash + Clone,
{
  v.iter().enumerate().map(|(i, item)| (item, i)).collect()
}

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
pub fn count_crossings(swappable_count: usize, static_count: usize, edges: &Vec<(usize, usize, usize)>) -> usize {
  // // Step 1
  let mut sorted_edges = edges.clone();
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

/**
 * Helper function for determining the optimal ordering while performing the swapping algo.
 * Assuming the right side of the bipartite graph stays locked, we can compute the number of edge crossings that
 * a pair of nodes (A, B) contributes in both orderings (A, B) and (B, A). This contribution does not actually depend
 * on any of the nodes inbetween, but of course swapping non-neighbouring pairs requires summing the contributions of
 * each pair that is swapped. Works as follows:
 *  1. For each left node count the cumulative number of edges to each right node in both directions
 *  2. For each pair of nodes (A, B) on the left side, count their contribution in both orders
 *    a. If B comes AFTER A then for each edge coming from A, then it crosses all edges from B that have a SMALLER right index
 *    b. If B comes BEFORE B then for each edge coming from A, then it crosses all edges from B that have a GREATER right index
 *
 * Step 2 can be done with a beautiful matrix product:
 * - PC[A, B] = Sum_j {W[A, j] * Cf[B, j]} - Sum_j {W[A, j] * Cb[B, j]}
 * - PC = W * Cf^T - W * Cb^T
 * - PC = W * (Cf - Cb)^T := W * C^T
 * - PC^T = C * W^T
 */
fn count_pair_crossings(swappable_count: usize, static_count: usize, edges: &Vec<(usize, usize, usize)>) -> Vec<f64> {
  let mut weights: Vec<f64> = vec![0.; swappable_count * static_count];
  for (swappable_id, static_id, weight) in edges {
    weights[static_id * swappable_count + swappable_id] = *weight as f64;
  }

  // Step 1.
  // These sumulative sums are EXCLUSIVE so the computation in step 2 is simpler.
  let mut cumulative_weights_f: Vec<f64> = vec![0.; swappable_count * static_count];
  let mut cumulative_weights_b: Vec<f64> = vec![0.; swappable_count * static_count];
  let mut cumulative_weights: Vec<f64> = vec![0.; swappable_count * static_count];

  // TODO: Simplify?
  // This is in essence a matrix multiplication, but due to the symmetry of the right matrix it can happen in O(L*R)
  for swappable_id in 0..swappable_count {
    for static_id in 1..static_count {
      let index = swappable_id * static_count + static_id;
      let index_w = (static_id - 1) * swappable_count + swappable_id;
      cumulative_weights_f[index] = cumulative_weights_f[index - 1] + weights[index_w];
    }

    for static_id in (0..static_count - 1).rev() {
      let index = swappable_id * static_count + static_id;
      let index_w = (static_id + 1) * swappable_count + swappable_id;
      cumulative_weights_b[index] = cumulative_weights_b[index + 1] + weights[index_w];
    }

    for static_id in 0..static_count {
      let index = swappable_id * static_count + static_id;
      cumulative_weights[index] = cumulative_weights_b[index] - cumulative_weights_f[index];
    }
  }

  // Step 2.
  let mut pair_crossings: Vec<f64> = vec![0.; swappable_count * swappable_count];
  matmul(
    &cumulative_weights,
    &weights,
    &mut pair_crossings,
    swappable_count,
    static_count,
    swappable_count,
  );

  pair_crossings
}

fn swap_nodes(swappable_count: usize, pair_crossings: &[f64], max_iterations: usize, temperature: f64) -> Vec<usize> {
  let mut nodes = (0..swappable_count).collect_vec();

  // if crossings > 0 {
  for _ in 0..max_iterations {
    for j in 0..swappable_count - 1 {
      let (node_a, node_b) = (nodes[j], nodes[j + 1]);
      let contribution = pair_crossings[node_a * swappable_count + node_b];
      // println!("Nodes ({}, {}) have contrib {}", node_names[node_a], node_names[node_b], contribution);
      if contribution > 0. || ((contribution - 1.) / temperature).exp() > random::<f64>() {
        nodes[j] = node_b;
        nodes[j + 1] = node_a;
        // crossings -= contribution as i64;
        // println!("Swapped nodes, crossings = {}", crossings);
      }
    }

    // if crossings == 0 {
    //   break;
    // }
  }
  // }

  // crossings
  nodes
}

pub fn reduce_crossings<T>(
  swappable_nodes: &Vec<T>,
  static_nodes: &Vec<T>,
  edges: Vec<(T, T, usize)>,
  iterations: usize,
  temperature: f64,
) -> Vec<T>
where
  T: Eq + Hash + Clone + Display + Debug,
{
  let index_swappable = invert_vec(swappable_nodes);
  let index_static = invert_vec(static_nodes);

  let mapped_edges = edges
    .iter()
    .map(|(l, r, w)| (index_swappable[l], index_static[r], *w))
    .collect_vec();

  let pairwise_matrix = count_pair_crossings(swappable_nodes.len(), static_nodes.len(), &mapped_edges);
  let new_indices = swap_nodes(swappable_nodes.len(), &pairwise_matrix, iterations, temperature);

  new_indices.iter().map(|l| swappable_nodes[*l].clone()).collect_vec()
}
