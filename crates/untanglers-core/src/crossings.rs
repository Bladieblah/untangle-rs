use std::fmt::{Debug, Display};
use std::hash::Hash;

use crate::count_crossings::count_crossings;
use crate::mapping::swap_edges;
use crate::reducer;

#[derive(Copy, Clone, Debug)]
pub enum Side {
  Left,
  Right,
}

pub struct Crossings<T>
where
  T: Eq + Hash + Clone + Display + Debug,
{
  nodes_left: Vec<T>,
  nodes_right: Vec<T>,
  edges: Vec<(T, T, usize)>,
  inverted_edges: Vec<(T, T, usize)>,
}

impl<T> Crossings<T>
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
    );
    new_count
  }

  // pub fn cooldown_side(&mut self, start_temp: f64, end_temp: f64, steps: usize, iterations: usize, side: Side) -> i64 {
  //   let mut temp = start_temp;
  //   let delta_t = (end_temp / start_temp).powf(1. / (steps as f64));
  //   let mut crossings = 0;
  //   let pair_crossings = self.count_pair_crossings(side);

  //   for _ in 0..steps {
  //     crossings = self._swap_nodes(iterations, temp, &pair_crossings, side);
  //     temp *= delta_t;
  //   }

  //   crossings
  // }
}
