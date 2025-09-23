use std::fmt::{Debug, Display};
use std::hash::Hash;

use crate::error::OptimizerError;
use crate::mapping::reorder_nodes;
use crate::optimizer::Optimizer;
use crate::optimizer_ops::{impl_optimizer_ops, OptimizerInternalOps, OptimizerOps};
use crate::reducer::reduce_crossings;
use crate::utils::{validate_edge_uniqueness, validate_layers};

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
  pub fn new(node_layers: Vec<Vec<T>>, edges: Vec<Vec<(T, T, usize)>>) -> Result<Self, OptimizerError> {
    validate_layers(&node_layers, &edges)?;
    validate_edge_uniqueness(&edges)?;

    let optimizer = Optimizer::new(node_layers, edges);
    Ok(Self { optimizer })
  }

  pub fn swap_nodes(
    &mut self,
    temperature: f64,
    max_iterations: usize,
    layer_index: usize,
  ) -> Result<usize, OptimizerError> {
    let (nodes1, edges1, nodes2, edges2) = self.get_adjacent_layers(layer_index)?;

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

    Ok(new_count as usize)
  }

  pub fn cooldown(
    &mut self,
    start_temp: f64,
    end_temp: f64,
    steps: usize,
    max_iterations: usize,
    layer_index: usize,
  ) -> Result<usize, OptimizerError> {
    let (nodes1, edges1, nodes2, edges2) = self.get_adjacent_layers(layer_index)?;

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

    Ok(new_count as usize)
  }

  pub fn optimize(
    &mut self,
    start_temp: f64,
    end_temp: f64,
    steps: usize,
    max_iterations: usize,
    passes: usize,
  ) -> Result<usize, OptimizerError> {
    for _pass in 0..passes {
      for i in 0..self.optimizer.node_layers.len() {
        self.cooldown(start_temp, end_temp, steps, max_iterations, i)?;
      }
    }

    Ok(self.count_crossings())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::utils::*;

  #[test]
  fn test_swap_nodes() {
    let n = 200;

    let (nodes, edges) = gen_multi_graph(7, n).unwrap();
    let mut optimizer = LayoutOptimizer::new(nodes, edges).unwrap();
    let start_crossings = optimizer.count_crossings();
    let end_crossings = timeit("Optimize", || optimizer.swap_nodes(1., 200, 3)).unwrap();

    assert!(start_crossings > end_crossings);
    assert!(end_crossings > 0);

    let real_crossings = optimizer.count_layer_crossings(3).unwrap();
    assert_eq!(end_crossings, real_crossings);
  }

  #[test]
  fn test_cooldown() {
    let n = 200;

    let (nodes, edges) = gen_multi_graph(7, n).unwrap();
    let mut optimizer = LayoutOptimizer::new(nodes, edges).unwrap();
    let start_crossings = optimizer.count_crossings();
    let end_crossings = timeit("Optimize", || optimizer.cooldown(1., 0.1, 5, 200, 3)).unwrap();

    println!("Improved from {} to {}", start_crossings, end_crossings);
    assert!(start_crossings > end_crossings);
    assert!(end_crossings > 0);

    let real_crossings = optimizer.count_layer_crossings(3).unwrap();
    assert_eq!(end_crossings, real_crossings);
  }

  #[test]
  fn test_optimize() {
    let n = 200;

    let (nodes, edges) = gen_multi_graph(7, n).unwrap();
    let mut optimizer = LayoutOptimizer::new(nodes, edges).unwrap();
    let start_crossings = optimizer.count_crossings();
    let end_crossings = timeit("Optimize", || optimizer.optimize(1., 0.1, 5, 200, 20)).unwrap();

    println!("Improved from {} to {}", start_crossings, end_crossings);
    assert!(start_crossings > end_crossings);
    assert!(end_crossings > 0);
  }
}
