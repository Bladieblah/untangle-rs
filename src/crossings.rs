use itertools::Itertools;
use rand::random;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;

// #[derive(Clone)]
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
    U: Eq + Hash + Clone + Display,
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
   * Counts the number of edge crossings in a bipartite graph. This can be done in E * ln E time where E is the number of edges.
   * This approach only works if there is at most 1 edge per node-pair. The process works as follows:
   *  1. Sort the edges ascending by their <left node index>, <right node index>
   *  2. Iterate through the sorted edges
   *    a. A new edge crosses every existing edge that has a GREATER right node index (computed using a cumulative sum)
   *    b. The weights are counted multiplicatively (left as an exercise to the reader)
   *    c. Keep track of the number of edges that reach each right node
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
   */
  pub fn count_pair_crossings(&self) -> HashMap<(usize, usize), i64> {
    // Convert edges to a sparse matrix representation
    // <left_index, <right_index, weight>>
    let mut weights = HashMap::<&usize, HashMap<usize, usize>>::new();
    let index_right = Self::invert_vec(&self.right);
    for (left, right, weight) in &self.edges {
      weights.entry(left).or_default().insert(index_right[right], *weight);
    }

    // Step 1.
    // These sumulative sums are EXCLUSIVE so the computation in step 2 is simpler.
    let mut cumulative_weights_f = HashMap::<&usize, Vec<usize>>::new();
    let mut cumulative_weights_b = HashMap::<&usize, Vec<usize>>::new();

    for (left, value) in &weights {
      let mut c_weight_f = vec![0; self.size_right];
      for i in 1..self.size_right {
        c_weight_f[i] += c_weight_f[i - 1] + value.get(&(i - 1)).unwrap_or(&0_usize);
      }
      cumulative_weights_f.insert(left, c_weight_f);

      let mut c_weight_b = vec![0; self.size_right];
      for i in (0..self.size_right - 1).rev() {
        c_weight_b[i] += c_weight_b[i + 1] + value.get(&(i + 1)).unwrap_or(&0_usize);
      }
      cumulative_weights_b.insert(left, c_weight_b);
    }

    // Step 2.
    // This cartesion product only works because the constructor assigns consecutive ids
    let mut pair_crossings = HashMap::<(usize, usize), i64>::new();
    for (node_a, node_b) in (0..self.size_left).cartesian_product(0..self.size_left) {
      if node_a == node_b {
        continue;
      }

      // The crossings tuple contains the count in orders (A, B) and (B, A) respectively.
      let mut crossings = (0_usize, 0_usize);
      if let (Some(weigths_a), Some(c_weights_f), Some(c_weights_b)) = (
        weights.get(&node_a),
        cumulative_weights_f.get_mut(&node_b),
        cumulative_weights_b.get_mut(&node_b),
      ) {
        for j in 0..self.size_right {
          if let Some(w_a) = weigths_a.get(&j) {
            // c_weights_f and c_weights_b are excluding node j itself, see step 1.
            crossings.0 += w_a * c_weights_f[j]; // 2a.
            crossings.1 += w_a * c_weights_b[j]; // 2b.
          }
        }
      }

      // The crossings counts are anti-symmetric of course
      // Treat the amount of crossings as energy, so their difference represents a potential energy
      let contribution = (crossings.0 as i64) - (crossings.1 as i64);
      pair_crossings.insert((node_a, node_b), contribution);
      pair_crossings.insert((node_b, node_a), -contribution);
    }

    pair_crossings
  }

  pub fn swap_nodes(&mut self, max_iterations: usize, temperature: f64) {
    self._swap_nodes(max_iterations, temperature, self.count_pair_crossings());
  }

  pub fn _swap_nodes(&mut self, max_iterations: usize, temperature: f64, pair_crossings: HashMap<(usize, usize), i64>) {
    let mut crossings = self.count_crossings() as i64;
    if crossings > 0 {
      for _ in 0..max_iterations {
        for j in 0..self.size_left - 1 {
          let pair = (self.left[j], self.left[j + 1]);
          let contribution = pair_crossings[&pair];
          if contribution > 0 || ((contribution as f64 - 1.) / temperature).exp() > random::<f64>() {
            self.left[j] = pair.1;
            self.left[j + 1] = pair.0;
            crossings -= contribution;
          }
        }

        if crossings == 0 {
          break;
        }
      }
    }
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
  fn test_simple_graph() {
    env_logger::init();
    let nodes_left: Vec<u8> = vec![0, 1, 2];
    let nodes_right: Vec<u8> = vec![3, 4, 5];
    let edges: Vec<(u8, u8, usize)> = vec![(0, 5, 1), (1, 5, 2), (2, 4, 3)];
    let mut crossings = Crossings::<u8>::new(nodes_left, nodes_right, edges);
    assert_eq!(crossings.count_crossings(), 9);
    crossings.swap_nodes(10, 1e-3);
    assert_eq!(crossings.count_crossings(), 0);
  }

  #[test]
  fn test_difficult_graph() {
    let n = 20;
    let (nodes_left, nodes_right, edges) = generate_graph(n);

    let mut crossings = Crossings::new(nodes_left, nodes_right, edges);
    let start_crossings = crossings.count_crossings();
    let mut temp = 10.;
    let delta_t = 0.5;
    let max_iterations = 100000;
    let k = 10;
    for _ in 0..k {
      timeit("crossings", || crossings.swap_nodes(max_iterations, temp));
      temp *= delta_t;
    }
    let end_crossings = crossings.count_crossings();
    assert!(end_crossings < start_crossings);
  }
}
