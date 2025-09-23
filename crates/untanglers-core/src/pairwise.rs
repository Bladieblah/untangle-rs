use crate::utils::matmul;

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
pub fn get_pairwise_matrix(
  swappable_count: usize,
  static_count: usize,
  edges: &Vec<(usize, usize, usize)>,
) -> Vec<f64> {
  if (static_count) == 0 {
    return vec![0.; swappable_count * swappable_count];
  }

  let mut weights: Vec<f64> = vec![0.; swappable_count * static_count];
  for (swappable_id, static_id, weight) in edges {
    weights[static_id * swappable_count + swappable_id] = *weight as f64;
  }

  // Step 1.
  // These sumulative sums are EXCLUSIVE so the computation in step 2 is simpler.
  let mut cumulative_weights_f: Vec<f64> = vec![0.; swappable_count * static_count];
  let mut cumulative_weights_b: Vec<f64> = vec![0.; swappable_count * static_count];
  let mut cumulative_weights: Vec<f64> = vec![0.; swappable_count * static_count];

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
