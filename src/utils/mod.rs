use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::Rng;
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
