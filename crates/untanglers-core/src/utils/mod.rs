use itertools::Itertools;
use matrixmultiply::dgemm;
use rand::seq::SliceRandom;
use rand::Rng;
use std::fmt::Display;
use std::time::Instant;

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

type GraphType = (Vec<i32>, Vec<i32>, Vec<(i32, i32, usize)>);

pub fn generate_graph(n_nodes: i32) -> GraphType {
  let mut nodes_left = (0..n_nodes).collect_vec();
  let mut nodes_right = (0..n_nodes).collect_vec();
  let mut edges = Vec::<(i32, i32, usize)>::new();

  let mut l = 0;
  let mut r = 0;

  let mut rng = rand::rng();
  let k = 3;
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

  nodes_left.shuffle(&mut rng);
  nodes_right.shuffle(&mut rng);

  (nodes_left, nodes_right, edges)
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
