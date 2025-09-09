use std::fmt::{Debug, Display};
use std::hash::Hash;

use itertools::Itertools;

use crate::count_crossings::count_crossings;
use crate::mapping::swap_edges;
use crate::reducer::{self};

pub struct LayoutOptimizer<T>
where
  T: Eq + Hash + Clone + Display + Debug,
{
  node_layers: Vec<Vec<T>>,
  edges: Vec<Vec<(T, T, usize)>>,
  inverted_edges: Vec<Vec<(T, T, usize)>>,
}

impl<T> LayoutOptimizer<T>
where
  T: Eq + Hash + Clone + Display + Debug,
{
  pub fn new(node_layers: Vec<Vec<T>>, edges: Vec<Vec<(T, T, usize)>>) -> Self {
    let inverted_edges = edges.iter().map(|e| swap_edges(e)).collect_vec();

    let this = Self {
      node_layers,
      edges,
      inverted_edges,
    };

    log::debug!("Counted {} edge crossings.", this.count_crossings());

    this
  }

  pub fn count_crossings(&self) -> usize {
    let mut total_count = 0;

    for i in 0..self.node_layers.len() - 1 {
      total_count += count_crossings(&self.node_layers[i], &self.node_layers[i + 1], &self.edges[i])
    }

    total_count
  }

  // #[allow(dead_code)]
  // pub fn swap_nodes(&mut self, max_iterations: usize, temperature: f64, side: Side) -> i64 {
  //   match side {
  //     Side::Left => self.swap_nodes_left(max_iterations, temperature),
  //     Side::Right => self.swap_nodes_right(max_iterations, temperature),
  //   }
  // }

  #[allow(dead_code)]
  pub fn swap_nodes(&mut self, index: usize, max_iterations: usize, temperature: f64) -> i64 {
    if index >= self.node_layers.len() {
      panic!("Layer index out of range: {} > {}", index, self.node_layers.len() - 1);
    }

    let new_count: i64;
    if index == 0 {
      (self.node_layers[index], new_count) = reducer::reduce_crossings(
        &self.node_layers[index],
        &self.node_layers[index + 1],
        &self.edges[index],
        max_iterations,
        temperature,
        &None,
      );
    } else if index == self.node_layers.len() - 1 {
      (self.node_layers[index], new_count) = reducer::reduce_crossings(
        &self.node_layers[index],
        &self.node_layers[index - 1],
        &self.inverted_edges[index - 1],
        max_iterations,
        temperature,
        &None,
      );
    } else {
      (self.node_layers[index], new_count) = reducer::reduce_crossings2(
        &self.node_layers[index],
        &self.node_layers[index - 1],
        &self.inverted_edges[index - 1],
        &self.node_layers[index + 1],
        &self.edges[index],
        max_iterations,
        temperature,
        &None,
      );
    }
    new_count
  }

  // #[allow(dead_code)]
  // pub fn swap_nodes_left(&mut self, max_iterations: usize, temperature: f64) -> i64 {
  //   let new_count: i64;
  //   (self.nodes_left, new_count) = reducer::reduce_crossings(
  //     &self.nodes_left,
  //     &self.nodes_right,
  //     &self.edges,
  //     max_iterations,
  //     temperature,
  //     &None,
  //   );
  //   new_count
  // }

  // #[allow(dead_code)]
  // pub fn swap_nodes_right(&mut self, max_iterations: usize, temperature: f64) -> i64 {
  //   let new_count: i64;
  //   (self.nodes_right, new_count) = reducer::reduce_crossings(
  //     &self.nodes_right,
  //     &self.nodes_left,
  //     &self.inverted_edges,
  //     max_iterations,
  //     temperature,
  //     &None,
  //   );
  //   new_count
  // }

  // pub fn cooldown_side(&mut self, start_temp: f64, end_temp: f64, steps: usize, iterations: usize, side: Side) -> i64 {
  //   let new_count;
  //   match side {
  //     Side::Left => {
  //       (self.nodes_left, new_count) = cooldown(
  //         &self.nodes_left,
  //         &self.nodes_right,
  //         &self.edges,
  //         iterations,
  //         start_temp,
  //         end_temp,
  //         steps,
  //         &None,
  //       )
  //     }
  //     Side::Right => {
  //       (self.nodes_right, new_count) = cooldown(
  //         &self.nodes_right,
  //         &self.nodes_left,
  //         &self.inverted_edges,
  //         iterations,
  //         start_temp,
  //         end_temp,
  //         steps,
  //         &None,
  //       )
  //     }
  //   }
  //   new_count
  // }

  #[allow(dead_code)]
  pub fn cooldown(&mut self, index: usize, max_iterations: usize, start_temp: f64, end_temp: f64, steps: usize) -> i64 {
    if index >= self.node_layers.len() {
      panic!("Layer index out of range: {} > {}", index, self.node_layers.len() - 1);
    }

    let new_count: i64;
    if index == 0 {
      (self.node_layers[index], new_count) = reducer::cooldown(
        &self.node_layers[index],
        &self.node_layers[index + 1],
        &self.edges[index],
        max_iterations,
        start_temp,
        end_temp,
        steps,
        &None,
      );
    } else if index == self.node_layers.len() - 1 {
      (self.node_layers[index], new_count) = reducer::cooldown(
        &self.node_layers[index],
        &self.node_layers[index - 1],
        &self.edges[index - 1],
        max_iterations,
        start_temp,
        end_temp,
        steps,
        &None,
      );
    } else {
      (self.node_layers[index], new_count) = reducer::cooldown2(
        &self.node_layers[index],
        &self.node_layers[index - 1],
        &self.edges[index - 1],
        &self.node_layers[index + 1],
        &self.edges[index],
        max_iterations,
        start_temp,
        end_temp,
        steps,
        &None,
      );
    }
    new_count
  }

  pub fn optimize(&mut self, start_temp: f64, end_temp: f64, steps: usize, iterations: usize, passes: usize) -> i64 {
    let mut crossing_count = 0;
    for _ in 0..passes {
      for i in 0..self.node_layers.len() {
        crossing_count = self.cooldown(i, iterations, start_temp, end_temp, steps);
      }
    }

    crossing_count
  }

  pub fn get_nodes(&self) -> Vec<Vec<T>> {
    self.node_layers.clone()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::utils::*;

  #[test]
  fn test_optimize() {
    let n = 200;

    let (nodes_left, nodes_right, edges) = generate_graph(n);
    let mut optimizer = LayoutOptimizer::new(vec![nodes_left.clone(), nodes_right.clone()], vec![edges.clone()]);
    let start_crossings = optimizer.count_crossings() as i64;
    let end_crossings = timeit("Optimize", || optimizer.optimize(3., 0.01, 5, 200, 20));

    println!("Improved from {} to {}", start_crossings, end_crossings);
    assert!(start_crossings > end_crossings);
    assert!(end_crossings > 0);
  }
}
