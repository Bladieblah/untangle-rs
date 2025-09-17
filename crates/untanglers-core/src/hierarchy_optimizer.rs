use std::fmt::{Debug, Display};
use std::hash::Hash;

use crate::hierarchy::{groups_and_borders, reorder_hierarchy, reorder_node_groups};
use crate::mapping::reorder_nodes;
use crate::optimizer::Optimizer;
use crate::optimizer_ops::{impl_optimizer_ops, OptimizerInternalOps, OptimizerOps};
use crate::reducer::reduce_crossings;

type Hierarchy = Vec<Vec<Vec<usize>>>;

pub struct HierarchyOptimizer<T>
where
  T: Eq + Hash + Clone + Display + Debug,
{
  optimizer: Optimizer<T>,
  hierarchy: Hierarchy,
}

impl_optimizer_ops!(HierarchyOptimizer<T>);

impl<T> HierarchyOptimizer<T>
where
  T: Eq + Hash + Clone + Display + Debug,
{
  pub fn new(node_layers: Vec<Vec<T>>, edges: Vec<Vec<(T, T, usize)>>, hierarchy: Hierarchy) -> Self {
    let optimizer = Optimizer::new(node_layers, edges);
    Self { optimizer, hierarchy }
  }

  pub fn swap_nodes(
    &mut self,
    layer_index: usize,
    granularity: Option<usize>,
    max_iterations: usize,
    temperature: f64,
  ) -> i64 {
    let (nodes1, edges1, nodes2, edges2) = self.get_adjacent_layers(layer_index);
    let (groups, borders) = groups_and_borders(&self.hierarchy[layer_index], granularity);

    let (new_indices, new_count) = reduce_crossings(
      &self.optimizer.node_layers[layer_index],
      nodes1,
      edges1,
      nodes2,
      edges2,
      max_iterations,
      temperature,
      temperature,
      1,
      groups.clone(),
      borders,
    );

    match granularity {
      None => self.optimizer.node_layers[layer_index] = reorder_nodes(&self.optimizer.node_layers[layer_index], &new_indices),
      Some(granularity) => {
        self.optimizer.node_layers[layer_index] = reorder_node_groups(&self.optimizer.node_layers[layer_index], &groups.unwrap(), &new_indices);
        self.hierarchy[layer_index] = reorder_hierarchy(&self.hierarchy[layer_index], granularity, &new_indices);
      }
    }

    new_count
  }

  pub fn cooldown(
    &mut self,
    start_temp: f64,
    end_temp: f64,
    steps: usize,
    max_iterations: usize,
    layer_index: usize,
    granularity: Option<usize>,
  ) -> i64 {
    let (nodes1, edges1, nodes2, edges2) = self.get_adjacent_layers(layer_index);
    let (groups, borders) = groups_and_borders(&self.hierarchy[layer_index], granularity);

    let (new_indices, new_count) = reduce_crossings(
      &self.optimizer.node_layers[layer_index],
      nodes1,
      edges1,
      nodes2,
      edges2,
      max_iterations,
      start_temp,
      end_temp,
      steps,
      groups.clone(),
      borders,
    );

    match granularity {
      None => self.optimizer.node_layers[layer_index] = reorder_nodes(&self.optimizer.node_layers[layer_index], &new_indices),
      Some(granularity) => {
        self.optimizer.node_layers[layer_index] = reorder_node_groups(&self.optimizer.node_layers[layer_index], &groups.unwrap(), &new_indices);
        self.hierarchy[layer_index] = reorder_hierarchy(&self.hierarchy[layer_index], granularity, &new_indices);
      }
    }

    new_count
  }

  pub fn optimize(
    &mut self,
    start_temp: f64,
    end_temp: f64,
    steps: usize,
    max_iterations: usize,
    passes: usize,
  ) -> i64 {
    let mut crossing_count = 0;
    for _pass in 0..passes {
      for layer_index in 0..self.optimizer.node_layers.len() {
        for granularity in 0..self.hierarchy[layer_index].len() {
          self.cooldown(
            start_temp,
            end_temp,
            steps,
            max_iterations,
            layer_index,
            Some(granularity),
          );
        }
        crossing_count = self.cooldown(start_temp, end_temp, steps, max_iterations, layer_index, None);
      }
    }

    crossing_count
  }

  pub fn get_hierarchy(&self) -> Hierarchy {
    self.hierarchy.clone()
  }
}

#[cfg(test)]
mod tests {
  use std::collections::{HashMap, HashSet};

use super::*;
  use crate::{utils::*};

  fn get_clusters(hierarchy: &Hierarchy, layer_index: usize, nodes: &[Vec<i32>]) -> HashMap<usize, HashSet<i32>> {
    let mut clusters = HashMap::<usize, HashSet<i32>>::new();

    for granularity in 0..hierarchy[layer_index].len() {
      let mut group_start: usize = 0;
      for group_size in &hierarchy[layer_index][granularity] {
        let node_names: HashSet<i32> = (group_start..group_start + group_size).map(|i| nodes[layer_index][i]).collect();
        clusters.insert(*group_size, node_names);
        group_start += group_size;
      }
    }

    clusters
  }

  #[test]
  fn test_cooldown_hierarchy() {
    let n = 100;

    let hierarchy: Hierarchy = vec![vec![], vec![
      vec![10, 13, 7, 3, 2, 14, 20, 15, 16],
      vec![30, 19, 35, 16],
      vec![49, 51],
    ], vec![]];

    let (nodes, edges) = generate_multipartite_graph(3, n);
    let clusters = get_clusters(&hierarchy, 1, &nodes);
    let mut optimizer = HierarchyOptimizer::new(nodes, edges, hierarchy);
    let mut start_crossings = optimizer.count_crossings() as i64;

    for granularity in vec![
      None,
      Some(0_usize),
      Some(1_usize),
      Some(2_usize),
    ] {
      let end_crossings = timeit("Optimize", || optimizer.cooldown(1., 0.1, 5, 200, 1, granularity));

      assert_eq!(get_clusters(&optimizer.get_hierarchy(), 1, &optimizer.get_nodes()), clusters);
  
      assert!(start_crossings >= end_crossings, "{start_crossings} < {end_crossings}");
      println!("Improved from {} to {}", start_crossings, end_crossings);
      assert!(end_crossings > 0);
  
      let real_crossings = optimizer.count_layer_crossings(1);
      assert_eq!(end_crossings, real_crossings);
      start_crossings = end_crossings;
    }
  }

  #[test]
  fn test_optimize_hierarchy() {
    let n = 100;

    let hierarchy: Hierarchy = vec![vec![], vec![
      vec![10, 13, 7, 3, 2, 14, 20, 15, 16],
      vec![30, 19, 35, 16],
      vec![49, 51],
    ], vec![]];

    let (nodes, edges) = generate_multipartite_graph(3, n);
    let clusters = get_clusters(&hierarchy, 1, &nodes);
    let mut optimizer = HierarchyOptimizer::new(nodes, edges, hierarchy);
    let start_crossings = optimizer.count_crossings() as i64;
    
    let end_crossings = timeit("Optimize", || optimizer.optimize(1., 0.1, 5, 200, 20));

      assert_eq!(get_clusters(&optimizer.get_hierarchy(), 1, &optimizer.get_nodes()), clusters);

    println!("Improved from {} to {}", start_crossings, end_crossings);
    assert!(start_crossings > end_crossings);
    assert!(end_crossings > 0);
  }
}
