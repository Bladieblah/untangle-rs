#![allow(clippy::too_many_arguments)]
use itertools::Itertools;
use rand::random;
use std::fmt::{Debug, Display};
use std::hash::Hash;

use crate::count_crossings::_count_crossings;
use crate::mapping::map_edges;
use crate::pairwise::get_pairwise_matrix;
use crate::utils::add_matrix;

pub fn swap_nodes(
  swappable_count: usize,
  pairwise_matrix: &[f64],
  max_iterations: usize,
  temperature: f64,
  mut crossing_count: i64,
  nodes: Vec<usize>,
  borders: &Option<Vec<usize>>,
) -> (Vec<usize>, i64) {
  let mut new_nodes = nodes.clone();
  let indices = match borders {
    None => (0..swappable_count - 1).collect_vec(),
    Some(b) => (0..swappable_count - 1).filter(|i| !b.contains(i)).collect_vec(),
  };

  if crossing_count > 0 {
    for _ in 0..max_iterations {
      for j in &indices {
        let (node_a, node_b) = (new_nodes[*j], new_nodes[*j + 1]);
        let contribution = pairwise_matrix[node_a * swappable_count + node_b];
        if contribution > 0. || ((contribution - 1.) / temperature).exp() > random::<f64>() {
          new_nodes[*j] = node_b;
          new_nodes[*j + 1] = node_a;
          crossing_count -= contribution as i64;
          // println!("Swapped nodes {} <-> {} with contrib {} new count = {}", node_a, node_b, contribution, crossing_count);
        }
      }

      if crossing_count == 0 {
        break;
      }
    }
  }

  (new_nodes, crossing_count)
}

pub fn reduce_crossings<T>(
  swappable_nodes: &[T],
  static_nodes: &[T],
  edges: &[(T, T, usize)],
  iterations: usize,
  temperature: f64,
  borders: &Option<Vec<usize>>,
) -> (Vec<T>, i64)
where
  T: Eq + Hash + Clone + Display + Debug,
{
  let mapped_edges = map_edges(swappable_nodes, static_nodes, edges);

  let crossing_count = _count_crossings(static_nodes.len(), &mapped_edges);
  let pairwise_matrix = get_pairwise_matrix(swappable_nodes.len(), static_nodes.len(), &mapped_edges);

  let (new_indices, new_count) = swap_nodes(
    swappable_nodes.len(),
    &pairwise_matrix,
    iterations,
    temperature,
    crossing_count as i64,
    (0..swappable_nodes.len()).collect_vec(),
    borders,
  );

  (
    new_indices.iter().map(|l| swappable_nodes[*l].clone()).collect_vec(),
    new_count,
  )
}

pub fn reduce_crossings2<T>(
  swappable_nodes: &[T],
  static_nodes1: &[T],
  edges1: &[(T, T, usize)],
  static_nodes2: &[T],
  edges2: &[(T, T, usize)],
  iterations: usize,
  temperature: f64,
  borders: &Option<Vec<usize>>,
) -> (Vec<T>, i64)
where
  T: Eq + Hash + Clone + Display + Debug,
{
  let mapped_edges1 = map_edges(swappable_nodes, static_nodes1, edges1);
  let mapped_edges2 = map_edges(swappable_nodes, static_nodes2, edges2);

  let crossing_count =
    _count_crossings(static_nodes1.len(), &mapped_edges1) + _count_crossings(static_nodes2.len(), &mapped_edges2);
  let pairwise_matrix = add_matrix(
    &get_pairwise_matrix(swappable_nodes.len(), static_nodes1.len(), &mapped_edges2),
    &get_pairwise_matrix(swappable_nodes.len(), static_nodes2.len(), &mapped_edges2),
  );

  let (new_indices, new_count) = swap_nodes(
    swappable_nodes.len(),
    &pairwise_matrix,
    iterations,
    temperature,
    crossing_count as i64,
    (0..swappable_nodes.len()).collect_vec(),
    borders,
  );

  (
    new_indices.iter().map(|l| swappable_nodes[*l].clone()).collect_vec(),
    new_count,
  )
}

pub fn cooldown<T>(
  swappable_nodes: &[T],
  static_nodes: &[T],
  edges: &[(T, T, usize)],
  iterations: usize,
  start_temp: f64,
  end_temp: f64,
  steps: usize,
  borders: &Option<Vec<usize>>,
) -> (Vec<T>, i64)
where
  T: Eq + Hash + Clone + Display + Debug,
{
  let mut temperature = start_temp;
  let delta_t = (end_temp / start_temp).powf(1. / (steps as f64));

  let mapped_edges = map_edges(swappable_nodes, static_nodes, edges);

  let mut crossing_count = _count_crossings(static_nodes.len(), &mapped_edges) as i64;
  let pairwise_matrix = get_pairwise_matrix(swappable_nodes.len(), static_nodes.len(), &mapped_edges);
  let mut nodes = (0..swappable_nodes.len()).collect_vec();

  for _ in 0..steps {
    (nodes, crossing_count) = swap_nodes(
      swappable_nodes.len(),
      &pairwise_matrix,
      iterations,
      temperature,
      crossing_count as i64,
      nodes,
      borders,
    );
    temperature *= delta_t;
  }

  (
    nodes.iter().map(|l| swappable_nodes[*l].clone()).collect_vec(),
    crossing_count,
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{count_crossings::count_crossings, mapping::swap_edges, utils::*};

  #[test]
  fn test_simple_graph() {
    let nodes_left: Vec<u8> = vec![0, 1, 2, 10];
    let nodes_right: Vec<u8> = vec![3, 4, 5];
    let edges: Vec<(u8, u8, usize)> = vec![(0, 5, 1), (1, 5, 2), (2, 4, 3)];
    let mapped_edges = map_edges(&nodes_left, &nodes_right, &edges);

    // Test counting left side
    let expected_left: Vec<f64> = vec![0., 0., 3., 0., 0., 0., 6., 0., -3., -6., 0., 0., 0., 0., 0., 0.];
    assert_eq!(
      get_pairwise_matrix(nodes_left.len(), nodes_right.len(), &mapped_edges),
      expected_left
    );
    assert_eq!(count_crossings(&nodes_left, &nodes_right, &edges), 9);

    let (new_nodes, expected_count) = reduce_crossings(&nodes_left, &nodes_right, &edges, 10, 0., &None);
    let actual_count = count_crossings(&new_nodes, &nodes_right, &edges) as i64;
    assert_eq!(expected_count, actual_count);
    assert_eq!(actual_count, 0);

    // Test counting right side
    let inv_edges = swap_edges(&edges);
    let inv_mapped_edges = map_edges(&nodes_right, &nodes_left, &inv_edges);
    let expected_right: Vec<f64> = vec![0.0, 0.0, 0.0, 0.0, 0.0, 9.0, 0.0, -9.0, 0.0];
    assert_eq!(
      get_pairwise_matrix(nodes_right.len(), nodes_left.len(), &inv_mapped_edges),
      expected_right
    );
    assert_eq!(count_crossings(&nodes_right, &nodes_left, &inv_edges), 9);

    let (new_nodes, expected_count) = reduce_crossings(&nodes_right, &nodes_left, &inv_edges, 10, 0., &None);
    let actual_count = count_crossings(&nodes_left, &new_nodes, &edges) as i64;
    assert_eq!(expected_count, actual_count);
    assert_eq!(actual_count, 0);
  }

  #[test]
  fn test_difficult_graph2() {
    let n = 50;
    let temperature = 2.;
    let iterations = 1000;

    let (nodes_left, nodes_right, edges) = generate_graph(n);
    let swapped_edges = swap_edges(&edges);
    let start_crossings = count_crossings(&nodes_left, &nodes_right, &edges) as i64;

    let (new_order, mid_crossings) =
      reduce_crossings(&nodes_left, &nodes_right, &edges, iterations, temperature, &None);
    let (_, end_crossings) = reduce_crossings(&nodes_right, &new_order, &swapped_edges, iterations, temperature, &None);

    assert!(mid_crossings < start_crossings, "{mid_crossings} !< {start_crossings}");
    assert!(end_crossings < mid_crossings, "{end_crossings} !< {mid_crossings}");
  }
}
