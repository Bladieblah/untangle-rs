use std::fmt::{Debug, Display};
use std::hash::Hash;

use itertools::Itertools;

use crate::count_crossings::{_count_crossings, count_crossings};
use crate::mapping::{map_edges, reorder_nodes, swap_edges};
use crate::reducer::reduce_crossings_final;

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

  pub fn count_layer_crossings(&self, layer_index: usize) -> i64 {
    let (nodes1, edges1, nodes2, edges2) = self.get_adjacent_layers(layer_index);
    let mapped_edges1 = map_edges(&self.node_layers[layer_index], nodes1, edges1);
    let mut crossing_count = _count_crossings(nodes1.len(), &mapped_edges1) as i64;

    if let (Some(nodes2), Some(edges2)) = (nodes2, edges2) {
      let mapped_edges2 = map_edges(&self.node_layers[layer_index], nodes2, edges2);
      crossing_count += _count_crossings(nodes2.len(), &mapped_edges2) as i64;
    };

    crossing_count
  }

  pub fn count_crossings(&self) -> usize {
    let mut total_count = 0;

    for i in 0..self.node_layers.len() - 1 {
      total_count += count_crossings(&self.node_layers[i], &self.node_layers[i + 1], &self.edges[i])
    }

    total_count
  }

  #[allow(clippy::type_complexity)]
  fn get_adjacent_layers(
    &self,
    layer_index: usize,
  ) -> (&[T], &[(T, T, usize)], Option<&Vec<T>>, Option<&Vec<(T, T, usize)>>) {
    if layer_index >= self.node_layers.len() {
      panic!(
        "Layer index out of range: {} > {}",
        layer_index,
        self.node_layers.len() - 1
      );
    }

    if layer_index == 0 {
      (&self.node_layers[layer_index + 1], &self.edges[layer_index], None, None)
    } else if layer_index == self.node_layers.len() - 1 {
      (
        &self.node_layers[layer_index - 1],
        &self.inverted_edges[layer_index - 1],
        None,
        None,
      )
    } else {
      (
        &self.node_layers[layer_index - 1],
        &self.inverted_edges[layer_index - 1],
        Some(&self.node_layers[layer_index + 1]),
        Some(&self.edges[layer_index]),
      )
    }
  }

  #[allow(dead_code)]
  pub fn swap_nodes(&mut self, layer_index: usize, max_iterations: usize, temperature: f64) -> i64 {
    let (nodes1, edges1, nodes2, edges2) = self.get_adjacent_layers(layer_index);

    let (new_indices, new_count) = reduce_crossings_final(
      &self.node_layers[layer_index],
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

    self.node_layers[layer_index] = reorder_nodes(&self.node_layers[layer_index], &new_indices);

    new_count
  }

  #[allow(dead_code)]
  pub fn cooldown(
    &mut self,
    start_temp: f64,
    end_temp: f64,
    steps: usize,
    max_iterations: usize,
    layer_index: usize,
  ) -> i64 {
    let (nodes1, edges1, nodes2, edges2) = self.get_adjacent_layers(layer_index);

    let (new_indices, new_count) = reduce_crossings_final(
      &self.node_layers[layer_index],
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

    self.node_layers[layer_index] = reorder_nodes(&self.node_layers[layer_index], &new_indices);

    new_count
  }

  pub fn optimize(&mut self, start_temp: f64, end_temp: f64, steps: usize, max_iterations: usize, passes: usize) -> i64 {
    let mut crossing_count = 0;
    for pass in 0..passes {
      for i in 0..self.node_layers.len() {
        println!("Pass {} layer {}", pass, i);
        crossing_count = self.cooldown(start_temp, end_temp, steps, max_iterations, i);
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
  fn test_get_adjacent_layers() {
    let optimizer = LayoutOptimizer::new(
      vec![
        vec![1,2,3],
        vec![4,5,6],
        vec![7,8,9],
      ],
      vec![
        vec![(1,4,2), (1,5,1)],
        vec![(4,8,3), (6,7,4)],
      ]
    );

    let (nodes1, edges1, nodes2, edges2) = optimizer.get_adjacent_layers(0);
    assert_eq!(nodes1, vec![4,5,6]);
    assert_eq!(nodes2, None);
    assert_eq!(edges1, vec![(1,4,2), (1,5,1)]);
    assert_eq!(edges2, None);

    let (nodes1, edges1, nodes2, edges2) = optimizer.get_adjacent_layers(1);
    assert_eq!(nodes1, vec![1,2,3]);
    assert_eq!(nodes2, Some(&vec![7,8,9]));
    assert_eq!(edges1, vec![(4,1,2), (5,1,1)]);
    assert_eq!(edges2, Some(&vec![(4,8,3), (6,7,4)]));

    let (nodes1, edges1, nodes2, edges2) = optimizer.get_adjacent_layers(2);
    assert_eq!(nodes1, vec![4,5,6]);
    assert_eq!(nodes2, None);
    assert_eq!(edges1, vec![(8,4,3), (7,6,4)]);
    assert_eq!(edges2, None);

  }

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
