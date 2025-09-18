use std::fmt::{Debug, Display};
use std::hash::Hash;

use itertools::Itertools;

use crate::count_crossings::count_crossings;
use crate::error::OptimizerError;
use crate::mapping::swap_edges;

pub struct Optimizer<T>
where
  T: Eq + Hash + Clone + Display + Debug,
{
  pub node_layers: Vec<Vec<T>>,
  pub edges: Vec<Vec<(T, T, usize)>>,
  pub inverted_edges: Vec<Vec<(T, T, usize)>>,
}

impl<T> Optimizer<T>
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

  pub fn count_layer_crossings(&self, layer_index: usize) -> Result<usize, OptimizerError> {
    let mut crossing_count = 0;
    if layer_index < self.edges.len() {
      crossing_count += count_crossings(
        &self.node_layers[layer_index],
        &self.node_layers[layer_index + 1],
        &self.edges[layer_index],
      );

      if layer_index > 0 {
        crossing_count += count_crossings(
          &self.node_layers[layer_index - 1],
          &self.node_layers[layer_index],
          &self.edges[layer_index - 1],
        );
      }
    }

    Ok(crossing_count)
  }

  pub fn count_crossings(&self) -> usize {
    let mut total_count = 0;

    for i in 0..self.node_layers.len() - 1 {
      total_count += count_crossings(&self.node_layers[i], &self.node_layers[i + 1], &self.edges[i])
    }

    total_count
  }

  #[allow(clippy::type_complexity)]
  pub fn get_adjacent_layers(
    &self,
    layer_index: usize,
  ) -> Result<(&[T], &[(T, T, usize)], Option<&Vec<T>>, Option<&Vec<(T, T, usize)>>), OptimizerError> {
    if layer_index >= self.node_layers.len() {
      return Err(OptimizerError::InvalidLayer {
        layer_index,
        layer_count: self.node_layers.len(),
      });
    }

    Ok(if layer_index == 0 {
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
    })
  }

  pub fn get_nodes(&self) -> Vec<Vec<T>> {
    self.node_layers.clone()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_adjacent_layers() {
    let optimizer = Optimizer::new(
      vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]],
      vec![vec![(1, 4, 2), (1, 5, 1)], vec![(4, 8, 3), (6, 7, 4)]],
    );

    let (nodes1, edges1, nodes2, edges2) = optimizer.get_adjacent_layers(0).unwrap();
    assert_eq!(nodes1, vec![4, 5, 6]);
    assert_eq!(nodes2, None);
    assert_eq!(edges1, vec![(1, 4, 2), (1, 5, 1)]);
    assert_eq!(edges2, None);

    let (nodes1, edges1, nodes2, edges2) = optimizer.get_adjacent_layers(1).unwrap();
    assert_eq!(nodes1, vec![1, 2, 3]);
    assert_eq!(nodes2, Some(&vec![7, 8, 9]));
    assert_eq!(edges1, vec![(4, 1, 2), (5, 1, 1)]);
    assert_eq!(edges2, Some(&vec![(4, 8, 3), (6, 7, 4)]));

    let (nodes1, edges1, nodes2, edges2) = optimizer.get_adjacent_layers(2).unwrap();
    assert_eq!(nodes1, vec![4, 5, 6]);
    assert_eq!(nodes2, None);
    assert_eq!(edges1, vec![(8, 4, 3), (7, 6, 4)]);
    assert_eq!(edges2, None);
  }
}
