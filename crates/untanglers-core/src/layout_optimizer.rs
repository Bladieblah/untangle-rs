use std::fmt::{Debug, Display};
use std::hash::Hash;

use crate::mapping::reorder_nodes;
use crate::optimizer::Optimizer;
use crate::optimizer_ops::{impl_optimizer_ops, OptimizerInternalOps, OptimizerOps};
use crate::reducer::reduce_crossings;

pub struct LayoutOptimizer<T>
where
  T: Eq + Hash + Clone + Display + Debug,
{
  optimizer: Optimizer<T>,
}

impl_optimizer_ops!(LayoutOptimizer<T>);

impl<T> LayoutOptimizer<T>
where
  T: Eq + Hash + Clone + Display + Debug,
{
  pub fn new(node_layers: Vec<Vec<T>>, edges: Vec<Vec<(T, T, usize)>>) -> Self {
    let optimizer = Optimizer::new(node_layers, edges);
    Self { optimizer }
  }

  pub fn swap_nodes(&mut self, layer_index: usize, max_iterations: usize, temperature: f64) -> i64 {
    let (nodes1, edges1, nodes2, edges2) = self.get_adjacent_layers(layer_index);

    let (new_indices, new_count) = reduce_crossings(
      &self.optimizer.node_layers[layer_index],
      nodes1,
      edges1,
      nodes2,
      edges2,
      max_iterations,
      temperature,
      temperature,
      1,
      None,
      None,
    );

    self.optimizer.node_layers[layer_index] = reorder_nodes(&self.optimizer.node_layers[layer_index], &new_indices);

    new_count
  }

  pub fn cooldown(
    &mut self,
    start_temp: f64,
    end_temp: f64,
    steps: usize,
    max_iterations: usize,
    layer_index: usize,
  ) -> i64 {
    let (nodes1, edges1, nodes2, edges2) = self.get_adjacent_layers(layer_index);

    let (new_indices, new_count) = reduce_crossings(
      &self.optimizer.node_layers[layer_index],
      nodes1,
      edges1,
      nodes2,
      edges2,
      max_iterations,
      start_temp,
      end_temp,
      steps,
      None,
      None,
    );

    self.optimizer.node_layers[layer_index] = reorder_nodes(&self.optimizer.node_layers[layer_index], &new_indices);

    new_count
  }

  pub fn optimize(
    &mut self,
    start_temp: f64,
    end_temp: f64,
    steps: usize,
    max_iterations: usize,
    passes: usize,
  ) -> i64 {
    let mut crossing_count = 0;
    for _pass in 0..passes {
      for i in 0..self.optimizer.node_layers.len() {
        crossing_count = self.cooldown(start_temp, end_temp, steps, max_iterations, i);
      }
    }

    crossing_count
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::utils::*;

  #[test]
  fn test_cooldown() {
    let n = 200;

    let (nodes, edges) = generate_multipartite_graph(7, n);
    let mut optimizer = LayoutOptimizer::new(nodes, edges);
    let start_crossings = optimizer.count_crossings() as i64;
    let end_crossings = timeit("Optimize", || optimizer.cooldown(1., 0.1, 5, 200, 1));

    println!("Improved from {} to {}", start_crossings, end_crossings);
    assert!(start_crossings > end_crossings);
    assert!(end_crossings > 0);

    let real_crossings = optimizer.count_layer_crossings(1);
    assert_eq!(end_crossings, real_crossings);
  }

  #[test]
  fn test_optimize() {
    let n = 200;

    let (nodes, edges) = generate_multipartite_graph(7, n);
    let mut optimizer = LayoutOptimizer::new(nodes, edges);
    let start_crossings = optimizer.count_crossings() as i64;
    let end_crossings = timeit("Optimize", || optimizer.optimize(1., 0.1, 5, 200, 20));

    println!("Improved from {} to {}", start_crossings, end_crossings);
    assert!(start_crossings > end_crossings);
    assert!(end_crossings > 0);
  }
}
