use std::fmt::{Debug, Display};
use std::hash::Hash;

use crate::count_crossings::count_crossings;
use crate::mapping::swap_edges;
use crate::reducer::{self, cooldown};

#[derive(Copy, Clone, Debug)]
pub enum Side {
  Left,
  Right,
}

pub struct LayoutOptimizer<T>
where
  T: Eq + Hash + Clone + Display + Debug,
{
  nodes_left: Vec<T>,
  nodes_right: Vec<T>,
  edges: Vec<(T, T, usize)>,
  inverted_edges: Vec<(T, T, usize)>,
}

impl<T> LayoutOptimizer<T>
where
  T: Eq + Hash + Clone + Display + Debug,
{
  pub fn new(nodes_left: Vec<T>, nodes_right: Vec<T>, edges: Vec<(T, T, usize)>) -> Self {
    let inverted_edges = swap_edges(&edges);

    let this = Self {
      nodes_left,
      nodes_right,
      edges,
      inverted_edges,
    };

    log::debug!("Counted {} edge crossings.", this.count_crossings());

    this
  }

  pub fn count_crossings(&self) -> usize {
    count_crossings(&self.nodes_left, &self.nodes_right, &self.edges)
  }

  #[allow(dead_code)]
  pub fn swap_nodes(&mut self, max_iterations: usize, temperature: f64, side: Side) -> i64 {
    match side {
      Side::Left => self.swap_nodes_left(max_iterations, temperature),
      Side::Right => self.swap_nodes_right(max_iterations, temperature),
    }
  }

  #[allow(dead_code)]
  pub fn swap_nodes_left(&mut self, max_iterations: usize, temperature: f64) -> i64 {
    let new_count: i64;
    (self.nodes_left, new_count) = reducer::reduce_crossings(
      &self.nodes_left,
      &self.nodes_right,
      &self.edges,
      max_iterations,
      temperature,
      &None,
    );
    new_count
  }

  #[allow(dead_code)]
  pub fn swap_nodes_right(&mut self, max_iterations: usize, temperature: f64) -> i64 {
    let new_count: i64;
    (self.nodes_right, new_count) = reducer::reduce_crossings(
      &self.nodes_right,
      &self.nodes_left,
      &self.inverted_edges,
      max_iterations,
      temperature,
      &None,
    );
    new_count
  }

  pub fn cooldown_side(&mut self, start_temp: f64, end_temp: f64, steps: usize, iterations: usize, side: Side) -> i64 {
    let new_count;
    match side {
      Side::Left => {
        (self.nodes_left, new_count) = cooldown(
          &self.nodes_left,
          &self.nodes_right,
          &self.edges,
          iterations,
          start_temp,
          end_temp,
          steps,
          &None,
        )
      }
      Side::Right => {
        (self.nodes_right, new_count) = cooldown(
          &self.nodes_right,
          &self.nodes_left,
          &self.inverted_edges,
          iterations,
          start_temp,
          end_temp,
          steps,
          &None,
        )
      }
    }
    new_count
  }

  pub fn optimize(&mut self, start_temp: f64, end_temp: f64, steps: usize, iterations: usize, passes: usize) -> i64 {
    let mut crossing_count = 0;
    for _ in 0..passes {
      self.cooldown_side(start_temp, end_temp, steps, iterations, Side::Left);
      crossing_count = self.cooldown_side(start_temp, end_temp, steps, iterations, Side::Right);
    }

    crossing_count
  }

  pub fn get_nodes(&self) -> (Vec<T>, Vec<T>) {
    (self.nodes_left.clone(), self.nodes_right.clone())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::utils::*;

  #[test]
  fn test_optimize() {
    let n = 2000;

    let (nodes_left, nodes_right, edges) = generate_graph(n);
    let mut optimizer = LayoutOptimizer::new(nodes_left.clone(), nodes_right.clone(), edges.clone());
    let start_crossings = optimizer.count_crossings() as i64;
    let end_crossings = timeit("Optimize", || optimizer.optimize(3., 0.1, 5, 1000, 10));

    println!("Improved from {} to {}", start_crossings, end_crossings);
    assert!(start_crossings > end_crossings);
    assert!(end_crossings > 0);

    // let (new_order, mid_crossings) = reduce_crossings(&nodes_left, &nodes_right, &edges, iterations, temperature);
    // let (_, end_crossings) = reduce_crossings(&nodes_right, &new_order, &swapped_edges, iterations, temperature);

    // assert!(mid_crossings < start_crossings, "{mid_crossings} !< {start_crossings}");
    // assert!(end_crossings < mid_crossings, "{end_crossings} !< {mid_crossings}");
  }
}
