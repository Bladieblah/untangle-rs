use itertools::Itertools;
use rand::random;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;

use crate::utils::matmul;

#[derive(Copy, Clone)]
pub enum Side {
  Left,
  Right,
}

pub struct Crossings<T>
where
  T: Eq + Hash + Clone + Display,
{
  nodes_left: Vec<T>,
  nodes_right: Vec<T>,
  left: Vec<usize>,
  right: Vec<usize>,
  edges: Vec<(usize, usize, usize)>,
  size_left: usize,
  size_right: usize,
}

impl<T> Crossings<T>
where
  T: Eq + Hash + Clone + Display,
{
  // Static methods

  fn invert_vec<U>(v: &[U]) -> HashMap<&U, usize>
  where
    U: Eq + Hash + Clone,
  {
    v.iter().enumerate().map(|(i, item)| (item, i)).collect()
  }

  pub fn new(nodes_left: Vec<T>, nodes_right: Vec<T>, edges: Vec<(T, T, usize)>) -> Self {
    let size_left = nodes_left.len();
    let size_right = nodes_right.len();

    let left = (0..size_left).collect_vec();
    let right = (0..size_right).collect_vec();

    let index_left = Self::invert_vec(&nodes_left);
    let index_right = Self::invert_vec(&nodes_right);

    let mapped_edges = edges
      .iter()
      .map(|(l, r, w)| (index_left[l], index_right[r], *w))
      .collect_vec();

    let this = Self {
      nodes_left,
      nodes_right,
      left,
      right,
      edges: mapped_edges,
      size_left,
      size_right,
    };

    log::debug!("Counted {} edge crossings.", this.count_crossings());

    this
  }

  // Instance methods

  fn get_node_indices(&self) -> (HashMap<&usize, usize>, HashMap<&usize, usize>) {
    let index_left = Self::invert_vec(&self.left);
    let index_right = Self::invert_vec(&self.right);

    (index_left, index_right)
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
  pub fn count_crossings(&self) -> usize {
    let (index_left, index_right) = self.get_node_indices();

    // Step 1
    let mut edges: Vec<(usize, usize, &usize)> = self
      .edges
      .iter()
      .map(|(l, r, w)| (index_left[l], index_right[r], w))
      .collect();
    edges.sort_unstable();

    let mut weights = vec![0_usize; self.size_right];
    let mut crossings = 0_usize;

    // Step 2
    for (_, right, weight) in edges {
      crossings += *weight * weights[right + 1..].iter().sum::<usize>(); // a. b.
      weights[right] += *weight; // c.
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
  pub fn count_pair_crossings(&self, side: Side) -> Vec<f64> {
    let (node_count, internal_size) = match side {
      Side::Left => (self.size_left, self.size_right),
      Side::Right => (self.size_right, self.size_left),
    };

    let mut weights: Vec<f64> = vec![0.; node_count * internal_size];
    match side {
      Side::Left => {
        for (left, right, weight) in &self.edges {
          weights[right * self.size_left + left] = *weight as f64;
        }
      }
      Side::Right => {
        for (left, right, weight) in &self.edges {
          weights[left * self.size_right + right] = *weight as f64;
        }
      }
    }

    // Step 1.
    // These sumulative sums are EXCLUSIVE so the computation in step 2 is simpler.
    let mut cumulative_weights_f: Vec<f64> = vec![0.; node_count * internal_size];
    let mut cumulative_weights_b: Vec<f64> = vec![0.; node_count * internal_size];
    let mut cumulative_weights: Vec<f64> = vec![0.; node_count * internal_size];

    // TODO: Simplify?
    // This is in essence a matrix multiplication, but due to the symmetry of the right matrix it can happen in O(L*R)
    for swappable_id in 0..node_count {
      for static_id in 1..internal_size {
        let index = swappable_id * internal_size + static_id;
        let index_w = (static_id - 1) * node_count + swappable_id;
        cumulative_weights_f[index] = cumulative_weights_f[index - 1] + weights[index_w];
      }

      for static_id in (0..internal_size - 1).rev() {
        let index = swappable_id * internal_size + static_id;
        let index_w = (static_id + 1) * node_count + swappable_id;
        cumulative_weights_b[index] = cumulative_weights_b[index + 1] + weights[index_w];
      }

      for static_id in (0..internal_size).rev() {
        let index = swappable_id * internal_size + static_id;
        cumulative_weights[index] = cumulative_weights_b[index] - cumulative_weights_f[index];
      }
    }

    // Step 2.
    let mut pair_crossings: Vec<f64> = vec![0.; node_count * node_count];
    matmul(
      &cumulative_weights,
      &weights,
      &mut pair_crossings,
      self.size_left,
      self.size_right,
      self.size_left,
    );

    pair_crossings
  }

  #[allow(dead_code)]
  pub fn swap_nodes(&mut self, max_iterations: usize, temperature: f64, side: Side) -> i64 {
    self._swap_nodes(max_iterations, temperature, &self.count_pair_crossings(side), side)
  }

  #[allow(dead_code)]
  pub fn swap_nodes_left(&mut self, max_iterations: usize, temperature: f64) -> i64 {
    self._swap_nodes(
      max_iterations,
      temperature,
      &self.count_pair_crossings(Side::Left),
      Side::Left,
    )
  }

  #[allow(dead_code)]
  pub fn swap_nodes_right(&mut self, max_iterations: usize, temperature: f64) -> i64 {
    self._swap_nodes(
      max_iterations,
      temperature,
      &self.count_pair_crossings(Side::Right),
      Side::Right,
    )
  }

  pub fn _swap_nodes(&mut self, max_iterations: usize, temperature: f64, pair_crossings: &[f64], side: Side) -> i64 {
    let mut crossings = self.count_crossings() as i64;

    let (nodes, node_count) = match side {
      Side::Left => (&mut self.left, self.size_left),
      Side::Right => (&mut self.left, self.size_right),
    };

    if crossings > 0 {
      for _ in 0..max_iterations {
        for j in 0..node_count - 1 {
          let (node_a, node_b) = (nodes[j], nodes[j + 1]);
          let contribution = pair_crossings[node_a * node_count + node_b];
          if contribution > 0. || ((contribution - 1.) / temperature).exp() > random::<f64>() {
            nodes[j] = node_b;
            nodes[j + 1] = node_a;
            crossings -= contribution as i64;
          }
        }

        if crossings == 0 {
          break;
        }
      }
    }

    crossings
  }

  #[allow(dead_code)]
  pub fn cooldown_left(&mut self, start_temp: f64, end_temp: f64, steps: usize, iterations: usize) -> i64 {
    self.cooldown_side(start_temp, end_temp, steps, iterations, Side::Left)
  }

  #[allow(dead_code)]
  pub fn cooldown_right(&mut self, start_temp: f64, end_temp: f64, steps: usize, iterations: usize) -> i64 {
    self.cooldown_side(start_temp, end_temp, steps, iterations, Side::Right)
  }

  pub fn cooldown_side(&mut self, start_temp: f64, end_temp: f64, steps: usize, iterations: usize, side: Side) -> i64 {
    let mut temp = start_temp;
    let delta_t = (end_temp / start_temp).powf(1. / (steps as f64));
    let mut crossings = 0;
    let pair_crossings = self.count_pair_crossings(side);

    for _ in 0..steps {
      crossings = self._swap_nodes(iterations, temp, &pair_crossings, side);
      temp *= delta_t;
    }

    crossings
  }

  pub fn get_nodes(&self) -> (Vec<T>, Vec<T>) {
    (
      self.left.iter().map(|l| self.nodes_left[*l].clone()).collect_vec(),
      self.right.iter().map(|r| self.nodes_right[*r].clone()).collect_vec(),
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::utils::*;

  #[test]
  fn test_pairwise_matrix() {
    let nodes_left: Vec<u8> = vec![0, 1, 2, 10];
    let nodes_right: Vec<u8> = vec![3, 4, 5];
    let edges: Vec<(u8, u8, usize)> = vec![(0, 5, 1), (1, 5, 2), (2, 4, 3)];

    let crossings = Crossings::<u8>::new(nodes_left.clone(), nodes_right.clone(), edges);

    let expected_left: Vec<f64> = vec![0., 0., 3., 0., 0., 0., 6., 0., -3., -6., 0., 0., 0., 0., 0., 0.];
    let expected_right: Vec<f64> = vec![0., 0., 3., 0., 0., 0., 6., 0., -3., -6., 0., 0., 0., 0., 0., 0.];
    assert_eq!(crossings.count_pair_crossings(Side::Left), expected_left);
    assert_eq!(crossings.count_pair_crossings(Side::Right), expected_right);
  }

  #[test]
  fn test_simple_graph() {
    let nodes_left: Vec<u8> = vec![0, 1, 2];
    let nodes_right: Vec<u8> = vec![3, 4, 5];
    let edges: Vec<(u8, u8, usize)> = vec![(0, 5, 1), (1, 5, 2), (2, 4, 3)];
    let mut crossings = Crossings::<u8>::new(nodes_left, nodes_right, edges);
    assert_eq!(crossings.count_crossings(), 9);
    crossings.swap_nodes(10, 1e-3, Side::Left);
    assert_eq!(crossings.count_crossings(), 0);
  }

  #[test]
  fn test_difficult_graph() {
    env_logger::init();
    let n = 50;
    let (nodes_left, nodes_right, edges) = generate_graph(n);

    let mut crossings = Crossings::new(nodes_left, nodes_right, edges.clone());
    let start_crossings = crossings.count_crossings();
    crossings.cooldown_left(10., 0.1, 5, 10000);
    let mid_crossings = crossings.count_crossings();
    assert!(mid_crossings < start_crossings);

    crossings.cooldown_right(10., 0.1, 5, 10000);
    let end_crossings = crossings.count_crossings();
    println!("{} -> {} -> {}", start_crossings, mid_crossings, end_crossings);
    assert!(end_crossings < mid_crossings, "{end_crossings} !< {mid_crossings}");

    println!("{} -> {} -> {}", start_crossings, mid_crossings, end_crossings)
  }
}
