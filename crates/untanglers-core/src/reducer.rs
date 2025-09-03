use itertools::Itertools;
use rand::random;
use std::fmt::{Debug, Display};
use std::hash::Hash;

use crate::count_crossings::_count_crossings;
use crate::mapping::map_edges;
use crate::pairwise::pairwise_matrix;

fn swap_nodes(
  swappable_count: usize,
  pair_crossings: &[f64],
  max_iterations: usize,
  temperature: f64,
  mut crossing_count: i64,
) -> (Vec<usize>, i64) {
  let mut nodes = (0..swappable_count).collect_vec();

  if crossing_count > 0 {
    for _ in 0..max_iterations {
      for j in 0..swappable_count - 1 {
        let (node_a, node_b) = (nodes[j], nodes[j + 1]);
        let contribution = pair_crossings[node_a * swappable_count + node_b];
        if contribution > 0. || ((contribution - 1.) / temperature).exp() > random::<f64>() {
          nodes[j] = node_b;
          nodes[j + 1] = node_a;
          crossing_count -= contribution as i64;
        }
      }

      if crossing_count == 0 {
        break;
      }
    }
  }

  (nodes, crossing_count)
}

pub fn reduce_crossings<T>(
  swappable_nodes: &[T],
  static_nodes: &[T],
  edges: &[(T, T, usize)],
  iterations: usize,
  temperature: f64,
) -> (Vec<T>, i64)
where
  T: Eq + Hash + Clone + Display + Debug,
{
  let mapped_edges = map_edges(swappable_nodes, static_nodes, edges);

  let crossing_count = _count_crossings(static_nodes.len(), &mapped_edges);
  let matrix = pairwise_matrix(swappable_nodes.len(), static_nodes.len(), &mapped_edges);
  let (new_indices, new_count) = swap_nodes(
    swappable_nodes.len(),
    &matrix,
    iterations,
    temperature,
    crossing_count as i64,
  );

  (
    new_indices.iter().map(|l| swappable_nodes[*l].clone()).collect_vec(),
    new_count,
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
      pairwise_matrix(nodes_left.len(), nodes_right.len(), &mapped_edges),
      expected_left
    );
    assert_eq!(count_crossings(&nodes_left, &nodes_right, &edges), 9);

    let (new_nodes, expected_count) = reduce_crossings(&nodes_left, &nodes_right, &edges, 10, 0.);
    let actual_count = count_crossings(&new_nodes, &nodes_right, &edges) as i64;
    assert_eq!(expected_count, actual_count);
    assert_eq!(actual_count, 0);

    // Test counting right side
    let inv_edges = swap_edges(&edges);
    let inv_mapped_edges = map_edges(&nodes_right, &nodes_left, &inv_edges);
    let expected_right: Vec<f64> = vec![0.0, 0.0, 0.0, 0.0, 0.0, 9.0, 0.0, -9.0, 0.0];
    assert_eq!(
      pairwise_matrix(nodes_right.len(), nodes_left.len(), &inv_mapped_edges),
      expected_right
    );
    assert_eq!(count_crossings(&nodes_right, &nodes_left, &inv_edges), 9);

    let (new_nodes, expected_count) = reduce_crossings(&nodes_right, &nodes_left, &inv_edges, 10, 0.);
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

    let (new_order, mid_crossings) = reduce_crossings(&nodes_left, &nodes_right, &edges, iterations, temperature);
    let (_, end_crossings) = reduce_crossings(&nodes_right, &new_order, &swapped_edges, iterations, temperature);

    assert!(mid_crossings < start_crossings, "{mid_crossings} !< {start_crossings}");
    assert!(end_crossings < mid_crossings, "{end_crossings} !< {mid_crossings}");
  }
}
