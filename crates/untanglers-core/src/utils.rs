use itertools::Itertools;
use matrixmultiply::dgemm;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::Rng;
use std::fmt::Display;
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
type GraphType = (Vec<Vec<i32>>, Vec<Vec<(i32, i32, usize)>>);

fn generate_edges(rng: &mut ThreadRng, n_nodes: i32) -> Vec<(i32, i32, usize)> {
  let mut l = 0;
  let mut r = 0;
  let k = 3;
  let mut edges = Vec::<(i32, i32, usize)>::new();
  while l < n_nodes - k && r < n_nodes - k {
    let dl = rng.random_range(1..k);
    for i in 0..dl {
      edges.push((l + i + 1, r, 1));
    }
    l += dl;

    let dr = rng.random_range(1..k + 1);
    for i in 0..dr {
      edges.push((l, r + i + 1, 1));
    }
    r += dr;
  }

  edges
}

pub fn generate_bipartite_graph(n_nodes: i32) -> BipartiteGraphType {
  let mut nodes_left = (0..n_nodes).collect_vec();
  let mut nodes_right = (0..n_nodes).collect_vec();

  let mut rng = rand::rng();
  let edges = generate_edges(&mut rng, n_nodes);

  nodes_left.shuffle(&mut rng);
  nodes_right.shuffle(&mut rng);

  (nodes_left, nodes_right, edges)
}

pub fn generate_multipartite_graph(n_layers: i32, n_nodes: i32) -> GraphType {
  let mut nodes = (0..n_layers).map(|_l| (0..n_nodes).collect_vec()).collect_vec();

  let mut rng = rand::rng();
  let edges = (0..n_layers - 1)
    .map(|_l| generate_edges(&mut rng, n_nodes))
    .collect_vec();

  (0..n_layers).for_each(|l| nodes[l as usize].shuffle(&mut rng));

  (nodes, edges)
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
where T: Clone + Display + Eq,
{
  if edges.len() != nodes.len() - 1 {
    return Err(OptimizerError::EdgeLayerMismatch { edges: edges.len(), layers: nodes.len() });
  }

  for layer_index in 0..edges.len() {
    for (node_a, node_b, _) in &edges[layer_index] {
      if !nodes[layer_index].contains(node_a) {
        return Err(OptimizerError::MissingNode { node_name: node_a.clone().to_string(), layer_index: layer_index })
      }
      if !nodes[layer_index + 1].contains(node_b) {
        return Err(OptimizerError::MissingNode { node_name: node_b.clone().to_string(), layer_index: layer_index + 1 })
      }
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
}
