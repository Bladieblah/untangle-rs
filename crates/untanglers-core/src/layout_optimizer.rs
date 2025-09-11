use std::fmt::{Debug, Display};
use std::hash::Hash;

use itertools::Itertools;

use crate::count_crossings::count_crossings;
use crate::mapping::swap_edges;
use crate::reducer::{self};

pub struct LayoutOptimizer<T>
where
  T: Eq + Hash + Clone + Display + Debug,
{
  node_layers: Vec<Vec<T>>,
  edges: Vec<Vec<(T, T, usize)>>,
  inverted_edges: Vec<Vec<(T, T, usize)>>,
}

impl<T> LayoutOptimizer<T>
where
  T: Eq + Hash + Clone + Display + Debug,
{
  pub fn new(node_layers: Vec<Vec<T>>, edges: Vec<Vec<(T, T, usize)>>) -> Self {
    let inverted_edges = edges.iter().map(|e| swap_edges(e)).collect_vec();

    Self {
      node_layers,
      edges,
      inverted_edges,
    }
  }

  pub fn count_crossings(&self) -> usize {
    let mut total_count = 0;

    for i in 0..self.node_layers.len() - 1 {
      total_count += count_crossings(&self.node_layers[i], &self.node_layers[i + 1], &self.edges[i])
    }

    total_count
  }

  #[allow(dead_code)]
  pub fn swap_nodes(&mut self, layer_index: usize, max_iterations: usize, temperature: f64) -> i64 {
    if layer_index >= self.node_layers.len() {
      panic!(
        "Layer index out of range: {} > {}",
        layer_index,
        self.node_layers.len() - 1
      );
    }

    let new_count: i64;
    if layer_index == 0 {
      (self.node_layers[layer_index], new_count) = reducer::reduce_crossings(
        &self.node_layers[layer_index],
        &self.node_layers[layer_index + 1],
        &self.edges[layer_index],
        max_iterations,
        temperature,
        &None,
      );
    } else if layer_index == self.node_layers.len() - 1 {
      (self.node_layers[layer_index], new_count) = reducer::reduce_crossings(
        &self.node_layers[layer_index],
        &self.node_layers[layer_index - 1],
        &self.inverted_edges[layer_index - 1],
        max_iterations,
        temperature,
        &None,
      );
    } else {
      (self.node_layers[layer_index], new_count) = reducer::reduce_crossings2(
        &self.node_layers[layer_index],
        &self.node_layers[layer_index - 1],
        &self.inverted_edges[layer_index - 1],
        &self.node_layers[layer_index + 1],
        &self.edges[layer_index],
        max_iterations,
        temperature,
        &None,
      );
    }
    new_count
  }

  #[allow(dead_code)]
  pub fn cooldown(
    &mut self,
    layer_index: usize,
    max_iterations: usize,
    start_temp: f64,
    end_temp: f64,
    steps: usize,
  ) -> i64 {
    if layer_index >= self.node_layers.len() {
      panic!(
        "Layer index out of range: {} > {}",
        layer_index,
        self.node_layers.len() - 1
      );
    }

    let new_count: i64;
    if layer_index == 0 {
      (self.node_layers[layer_index], new_count) = reducer::cooldown(
        &self.node_layers[layer_index],
        &self.node_layers[layer_index + 1],
        &self.edges[layer_index],
        max_iterations,
        start_temp,
        end_temp,
        steps,
        &None,
      );
    } else if layer_index == self.node_layers.len() - 1 {
      (self.node_layers[layer_index], new_count) = reducer::cooldown(
        &self.node_layers[layer_index],
        &self.node_layers[layer_index - 1],
        &self.edges[layer_index - 1],
        max_iterations,
        start_temp,
        end_temp,
        steps,
        &None,
      );
    } else {
      (self.node_layers[layer_index], new_count) = reducer::cooldown2(
        &self.node_layers[layer_index],
        &self.node_layers[layer_index - 1],
        &self.edges[layer_index - 1],
        &self.node_layers[layer_index + 1],
        &self.edges[layer_index],
        max_iterations,
        start_temp,
        end_temp,
        steps,
        &None,
      );
    }
    new_count
  }

  pub fn optimize(&mut self, start_temp: f64, end_temp: f64, steps: usize, iterations: usize, passes: usize) -> i64 {
    let mut crossing_count = 0;
    for _ in 0..passes {
      for i in 0..self.node_layers.len() {
        crossing_count = self.cooldown(i, iterations, start_temp, end_temp, steps);
      }
    }

    crossing_count
  }

  pub fn get_nodes(&self) -> Vec<Vec<T>> {
    self.node_layers.clone()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::utils::*;

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
