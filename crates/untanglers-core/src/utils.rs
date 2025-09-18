use itertools::Itertools;
use matrixmultiply::dgemm;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashSet;
use std::fmt::Display;
use std::hash::Hash;
use std::time::Instant;

use crate::error::OptimizerError;

pub fn timeit<F, R>(label: &str, f: F) -> R
where
  F: FnOnce() -> R,
{
  let start = Instant::now();
  let result = f();
  let elapsed = start.elapsed();
  println!("[{label}] took {:.3?}", elapsed);
  result
}

type BipartiteGraphType = (Vec<i32>, Vec<i32>, Vec<(i32, i32, usize)>);
pub type GraphType = (Vec<Vec<i32>>, Vec<Vec<(i32, i32, usize)>>);

fn generate_edges(
  rng: &mut ThreadRng,
  n_nodes1: usize,
  n_nodes2: usize,
  start1: i32,
  start2: i32,
) -> Vec<(i32, i32, usize)> {
  if n_nodes1 == 0 || n_nodes2 == 0 {
    return vec![];
  }

  let mut l = 0;
  let mut r = 0;
  let mut edges = Vec::<(i32, i32, usize)>::new();
  let k: usize = 3;
  edges.push((start1, start2, 1));
  while l < n_nodes1 - 1 || r < n_nodes2 - 1 {
    let dl = (n_nodes1 - l - 1).min(rng.random_range(1..k));

    for i in 1..=dl {
      edges.push((start1 + (l + i) as i32, start2 + r as i32, 1));
    }
    l += dl;

    let dr = (n_nodes2 - r - 1).min(rng.random_range(1..k));
    for i in 1..=dr {
      edges.push((start1 + l as i32, start2 + (r + i) as i32, 1));
    }
    r += dr;
  }

  edges
}

pub fn generate_bipartite_graph(n_nodes: usize) -> BipartiteGraphType {
  let mut nodes_left = (0..n_nodes as i32).collect_vec();
  let mut nodes_right: Vec<i32> = (n_nodes as i32..(n_nodes * 2) as i32).collect_vec();

  let mut rng = rand::rng();
  let edges = generate_edges(&mut rng, n_nodes, n_nodes, 0, n_nodes as i32);

  nodes_left.shuffle(&mut rng);
  nodes_right.shuffle(&mut rng);

  validate_layers(&[nodes_left.clone(), nodes_right.clone()], std::slice::from_ref(&edges)).unwrap();
  validate_edge_uniqueness(std::slice::from_ref(&edges)).unwrap();

  (nodes_left, nodes_right, edges)
}

pub fn gen_multi_graph(n_layers: usize, n_nodes: usize) -> Result<GraphType, OptimizerError> {
  generate_multipartite_graph(vec![n_nodes; n_layers])
}

pub fn generate_multipartite_graph(n_nodes: Vec<usize>) -> Result<GraphType, OptimizerError> {
  let n_layers = n_nodes.len();
  let starts = (0..n_layers)
    .map(|l| n_nodes[0..l].iter().sum::<usize>() as i32)
    .collect_vec();
  let mut nodes = (0..n_layers)
    .map(|l| (starts[l]..(starts[l] + n_nodes[l] as i32)).collect_vec())
    .collect_vec();

  let mut rng = rand::rng();
  let edges = (0..n_layers - 1)
    .map(|l| {
      generate_edges(
        &mut rng,
        n_nodes[l],
        n_nodes[l + 1],
        n_nodes[0..l].iter().sum::<usize>() as i32,
        n_nodes[0..l + 1].iter().sum::<usize>() as i32,
      )
    })
    .collect_vec();

  (0..n_layers).for_each(|l| nodes[l].shuffle(&mut rng));

  validate_layers(&nodes, &edges)?;
  validate_edge_uniqueness(&edges).unwrap();

  Ok((nodes, edges))
}

pub fn matmul(matrix_a: &[f64], matrix_b: &[f64], matrix_c: &mut [f64], m: usize, k: usize, n: usize) {
  unsafe {
    dgemm(
      m,
      k,
      n,
      1.0,
      matrix_a.as_ptr(),
      k as isize,
      1,
      matrix_b.as_ptr(),
      n as isize,
      1,
      0.0,
      matrix_c.as_mut_ptr(),
      n as isize,
      1,
    )
  }
}

pub fn validate_layers<T>(nodes: &[Vec<T>], edges: &[Vec<(T, T, usize)>]) -> Result<(), OptimizerError>
where
  T: Clone + Display + Eq,
{
  if edges.len() != nodes.len() - 1 {
    return Err(OptimizerError::EdgeLayerMismatch {
      edges: edges.len(),
      layers: nodes.len(),
    });
  }

  for layer_index in 0..edges.len() {
    for (node_a, node_b, _) in &edges[layer_index] {
      if !nodes[layer_index].contains(node_a) {
        return Err(OptimizerError::MissingNode {
          node_name: node_a.clone().to_string(),
          layer_index,
        });
      }
      if !nodes[layer_index + 1].contains(node_b) {
        return Err(OptimizerError::MissingNode {
          node_name: node_b.clone().to_string(),
          layer_index: layer_index + 1,
        });
      }
    }
  }

  Ok(())
}

pub fn validate_edge_uniqueness<T>(edges: &[Vec<(T, T, usize)>]) -> Result<(), OptimizerError>
where
  T: Clone + Display + Eq + Hash,
{
  for layer_index in 0..edges.len() {
    let edges = &edges[layer_index];
    let mut seen = HashSet::<(T, T)>::new();
    for (node_a, node_b, _) in edges {
      let tuple = (node_a.clone(), node_b.clone());
      if seen.contains(&tuple) {
        return Err(OptimizerError::DuplicateEdge {
          node_a: node_a.to_string(),
          node_b: node_b.to_string(),
          layer_index,
        });
      }
      seen.insert(tuple);
    }
  }

  Ok(())
}

#[allow(dead_code)]
pub fn print_matrix<T>(mat: &[T], rows: usize, cols: usize)
where
  T: Display,
{
  let top = format!("┌{}┐", "────────".repeat(cols));
  let bottom = format!("└{}┘", "────────".repeat(cols));

  println!("{top}");
  for i in 0..rows {
    print!("│");
    for j in 0..cols {
      print!("{:>7.2} ", mat[i * cols + j]);
    }
    println!("│");
  }
  println!("{bottom}");
}

pub fn add_matrix(matrix1: &[f64], matrix2: &[f64]) -> Vec<f64> {
  if matrix1.len() != matrix2.len() {
    panic!("Attempting to add matrices of different sizes");
  }
  (0..matrix1.len()).map(|i| matrix1[i] + matrix2[i]).collect_vec()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_add_matrix() {
    let mat1 = vec![0., 1., 0., 0., -1., 0., 3., 4., 0., -3., 0., 2., 0., -4., -2., 0.];
    let mat2 = vec![0., -1., 1., 0., 1., 0., 0., 2., -1., 0., 0., 1., 0., -2., -1., 0.];
    let result = vec![0., 0., 1., 0., 0., 0., 3., 6., -1., -3., 0., 3., 0., -6., -3., 0.];

    assert_eq!(add_matrix(&mat1, &mat2), result);
  }

  #[test]
  fn test_edge_uniqueness() {
    let valid_edges = vec![(0, 2, 0), (0, 3, 0), (1, 3, 0), (1, 4, 0), (2, 4, 0), (2, 5, 0)];

    assert!(validate_edge_uniqueness(&[valid_edges]).is_ok());

    let invalid_edges = vec![
      (0, 2, 0),
      (0, 3, 0),
      (1, 3, 0),
      (1, 4, 0),
      (2, 4, 0),
      (2, 4, 0),
      (2, 5, 0),
    ];

    let result = validate_edge_uniqueness(&[invalid_edges]);
    match result {
      Err(OptimizerError::DuplicateEdge {
        node_a,
        node_b,
        layer_index,
      }) => {
        assert_eq!(node_a, "2".to_string());
        assert_eq!(node_b, "4".to_string());
        assert_eq!(layer_index, 0);
      }
      Err(other) => panic!("Unexpected error: {}", other),
      Ok(_) => panic!("Expected an error"),
    }
  }
}
