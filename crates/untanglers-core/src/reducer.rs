#![allow(clippy::too_many_arguments)]
use itertools::Itertools;
use rand::random;
use std::fmt::{Debug, Display};
use std::hash::Hash;

use crate::aggregation::aggregate_pairwise_matrix;
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

  if swappable_count == 0 {
    return (new_nodes, crossing_count);
  }

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
        }

        if crossing_count < 0 {
          println!(
            "[ERROR] Swapped nodes {} <-> {} with contrib {} new count = {}",
            node_a, node_b, contribution, crossing_count
          );
          panic!("Crossing count turned negative: {}", crossing_count);
        }
      }

      if crossing_count == 0 {
        break;
      }
    }
  }

  (new_nodes, crossing_count)
}

fn matrix_and_count<T>(
  swappable_nodes: &[T],
  static_nodes1: &[T],
  edges1: &[(T, T, usize)],
  static_nodes2: Option<&Vec<T>>,
  edges2: Option<&Vec<(T, T, usize)>>,
) -> (i64, Vec<f64>)
where
  T: Eq + Hash + Clone + Display + Debug,
{
  let mapped_edges1 = map_edges(swappable_nodes, static_nodes1, edges1);
  let mut crossing_count = _count_crossings(static_nodes1.len(), &mapped_edges1) as i64;
  let mut pairwise_matrix = get_pairwise_matrix(swappable_nodes.len(), static_nodes1.len(), &mapped_edges1);

  if let (Some(static_nodes2), Some(edges2)) = (static_nodes2, edges2) {
    let mapped_edges2 = map_edges(swappable_nodes, static_nodes2, edges2);
    crossing_count += _count_crossings(static_nodes2.len(), &mapped_edges2) as i64;
    pairwise_matrix = add_matrix(
      &pairwise_matrix,
      &get_pairwise_matrix(swappable_nodes.len(), static_nodes2.len(), &mapped_edges2),
    );
  };

  (crossing_count, pairwise_matrix)
}

pub fn reduce_crossings<T>(
  swappable_nodes: &[T],
  static_nodes1: &[T],
  edges1: &[(T, T, usize)],
  static_nodes2: Option<&Vec<T>>,
  edges2: Option<&Vec<(T, T, usize)>>,
  max_iterations: usize,
  start_temp: f64,
  end_temp: f64,
  temp_steps: usize,
  groups: Option<Vec<usize>>,
  borders: Option<Vec<usize>>,
) -> (Vec<usize>, i64)
where
  T: Eq + Hash + Clone + Display + Debug,
{
  let (mut crossing_count, mut pairwise_matrix) =
    matrix_and_count(swappable_nodes, static_nodes1, edges1, static_nodes2, edges2);

  let swappable_count = match groups {
    Some(groups) => {
      pairwise_matrix = aggregate_pairwise_matrix(&pairwise_matrix, &groups);
      groups.len()
    }
    None => swappable_nodes.len(),
  };

  let mut temperature = start_temp;
  let delta_t: f64 = if temp_steps == 0 {
    0.
  } else {
    (end_temp / start_temp).powf(1. / (temp_steps as f64 - 1.))
  };
  let mut new_indices = (0..swappable_count).collect_vec();

  for _ in 0..temp_steps {
    (new_indices, crossing_count) = swap_nodes(
      swappable_count,
      &pairwise_matrix,
      max_iterations,
      temperature,
      crossing_count,
      new_indices,
      &borders,
    );
    temperature *= delta_t;
  }

  (new_indices, crossing_count)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    count_crossings::count_crossings,
    mapping::{reorder_nodes, swap_edges},
    utils::generate_bipartite_graph,
  };

  #[test]
  fn test_middle_layer() {
    let (crossing_count, pairwise_matrix) = matrix_and_count(
      &[4, 5, 6],
      &[1, 2, 3],
      &[(4, 1, 2), (5, 1, 1), (4, 2, 1), (6, 3, 10)],
      Some(&vec![7, 8, 9]),
      Some(&vec![(4, 8, 3), (5, 7, 2), (6, 9, 5)]),
    );

    assert_eq!(crossing_count, 7);

    let expected_matrix = vec![0., 7., -45., -7., 0., -20., 45., 20., 0.];
    assert_eq!(pairwise_matrix, expected_matrix);

    let (new_nodes, new_count) = swap_nodes(3, &pairwise_matrix, 1, 1e-5, crossing_count, vec![0, 1, 2], &None);
    assert_eq!(new_count, 0);
    assert_eq!(new_nodes, vec![1, 0, 2]);

    let (crossing_count, pairwise_matrix) = matrix_and_count(
      &[5, 4, 6],
      &[1, 2, 3],
      &[(4, 1, 2), (5, 1, 1), (4, 2, 1), (6, 3, 10)],
      Some(&vec![7, 8, 9]),
      Some(&vec![(4, 8, 3), (5, 7, 2), (6, 9, 5)]),
    );

    assert_eq!(crossing_count, 0);

    let expected_matrix = vec![0., -7., -20., 7., 0., -45., 20., 45., 0.];
    assert_eq!(pairwise_matrix, expected_matrix);
  }

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

    let (new_indices, expected_count) =
      reduce_crossings(&nodes_left, &nodes_right, &edges, None, None, 10, 0., 0., 1, None, None);

    let new_nodes = reorder_nodes(&nodes_left, &new_indices);
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

    let (new_indices, expected_count) = reduce_crossings(
      &nodes_right,
      &nodes_left,
      &inv_edges,
      None,
      None,
      10,
      0.,
      0.,
      1,
      None,
      None,
    );
    let new_nodes = reorder_nodes(&nodes_right, &new_indices);
    let actual_count: i64 = count_crossings(&nodes_left, &new_nodes, &edges) as i64;
    assert_eq!(expected_count, actual_count);
    assert_eq!(actual_count, 0);
  }

  #[test]
  fn test_difficult_graph() {
    let n = 50;
    let temperature = 2.;
    let iterations = 1000;

    let (nodes_left, nodes_right, edges) = generate_bipartite_graph(n);
    let swapped_edges = swap_edges(&edges);
    let start_crossings = count_crossings(&nodes_left, &nodes_right, &edges) as i64;

    assert_eq!(
      start_crossings,
      count_crossings(&nodes_right, &nodes_left, &swapped_edges) as i64
    );

    let (new_indices, mid_crossings) = reduce_crossings(
      &nodes_left,
      &nodes_right,
      &edges,
      None,
      None,
      iterations,
      temperature,
      temperature,
      1,
      None,
      None,
    );

    let new_nodes_left = reorder_nodes(&nodes_left, &new_indices);
    assert_eq!(
      mid_crossings,
      count_crossings(&new_nodes_left, &nodes_right, &edges) as i64
    );

    let (new_indices, end_crossings) = reduce_crossings(
      &nodes_right,
      &new_nodes_left,
      &swapped_edges,
      None,
      None,
      iterations,
      temperature,
      temperature / 10.,
      2,
      None,
      None,
    );

    let new_nodes_right = reorder_nodes(&nodes_right, &new_indices);

    assert!(mid_crossings < start_crossings, "{mid_crossings} !< {start_crossings}");
    assert!(mid_crossings > 0, "{mid_crossings} < 0");
    assert!(end_crossings < mid_crossings, "{end_crossings} !< {mid_crossings}");
    assert!(end_crossings > 0, "{end_crossings} < 0");
    assert_eq!(
      end_crossings,
      count_crossings(&new_nodes_left, &new_nodes_right, &edges) as i64
    );
  }

  #[test]
  fn test_empty_nodes() {
    let nodes_left: Vec<u8> = vec![0, 1, 2, 10];
    let nodes_right: Vec<u8> = vec![];
    let edges: Vec<(u8, u8, usize)> = vec![];

    assert_eq!(count_crossings(&nodes_left, &nodes_right, &edges), 0);

    let (_, expected_count) =
      reduce_crossings(&nodes_left, &nodes_right, &edges, None, None, 10, 0., 0., 1, None, None);

    assert_eq!(expected_count, 0);

    let (_, expected_count) =
      reduce_crossings(&nodes_right, &nodes_left, &edges, None, None, 10, 0., 0., 1, None, None);

    assert_eq!(expected_count, 0);
  }
}
