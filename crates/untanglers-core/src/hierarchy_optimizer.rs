use std::fmt::{Debug, Display};
use std::hash::Hash;

use crate::error::OptimizerError;
use crate::hierarchy::{groups_and_borders, reorder_hierarchy, reorder_node_groups, validate_hierarchy};
use crate::mapping::reorder_nodes;
use crate::optimizer::Optimizer;
use crate::optimizer_ops::{impl_optimizer_ops, OptimizerInternalOps, OptimizerOps};
use crate::reducer::reduce_crossings;
use crate::utils::{validate_edge_uniqueness, validate_layers};

pub type Hierarchy = Vec<Vec<Vec<usize>>>;

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
  pub fn new(
    node_layers: Vec<Vec<T>>,
    edges: Vec<Vec<(T, T, usize)>>,
    hierarchy: Hierarchy,
  ) -> Result<Self, OptimizerError> {
    if hierarchy.len() != node_layers.len() {
      return Err(OptimizerError::HierarchyMismatch {
        hierarchy: hierarchy.len(),
        layers: node_layers.len(),
      });
    }

    for layer_index in 0..hierarchy.len() {
      validate_hierarchy(layer_index, node_layers[layer_index].len(), &hierarchy[layer_index])?;
    }

    validate_layers(&node_layers, &edges)?;
    validate_edge_uniqueness(&edges)?;

    let optimizer = Optimizer::new(node_layers, edges);
    Ok(Self { optimizer, hierarchy })
  }

  pub fn swap_nodes(
    &mut self,
    temperature: f64,
    max_iterations: usize,
    layer_index: usize,
    granularity: Option<usize>,
  ) -> Result<usize, OptimizerError> {
    let (nodes1, edges1, nodes2, edges2) = self.get_adjacent_layers(layer_index)?;
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
      None => {
        self.optimizer.node_layers[layer_index] = reorder_nodes(&self.optimizer.node_layers[layer_index], &new_indices)
      }
      Some(granularity) => {
        self.optimizer.node_layers[layer_index] =
          reorder_node_groups(&self.optimizer.node_layers[layer_index], &groups.unwrap(), &new_indices);
        self.hierarchy[layer_index] = reorder_hierarchy(&self.hierarchy[layer_index], granularity, &new_indices);
      }
    }

    Ok(new_count as usize)
  }

  pub fn cooldown(
    &mut self,
    start_temp: f64,
    end_temp: f64,
    steps: usize,
    max_iterations: usize,
    layer_index: usize,
    granularity: Option<usize>,
  ) -> Result<usize, OptimizerError> {
    let (nodes1, edges1, nodes2, edges2) = self.get_adjacent_layers(layer_index)?;
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
      None => {
        self.optimizer.node_layers[layer_index] = reorder_nodes(&self.optimizer.node_layers[layer_index], &new_indices)
      }
      Some(granularity) => {
        self.optimizer.node_layers[layer_index] =
          reorder_node_groups(&self.optimizer.node_layers[layer_index], &groups.unwrap(), &new_indices);
        self.hierarchy[layer_index] = reorder_hierarchy(&self.hierarchy[layer_index], granularity, &new_indices);
      }
    }

    Ok(new_count as usize)
  }

  pub fn optimize(
    &mut self,
    start_temp: f64,
    end_temp: f64,
    steps: usize,
    max_iterations: usize,
    passes: usize,
  ) -> Result<usize, OptimizerError> {
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
          )?;
        }
        self.cooldown(start_temp, end_temp, steps, max_iterations, layer_index, None)?;
      }
    }

    Ok(self.count_crossings())
  }

  pub fn get_hierarchy(&self) -> Hierarchy {
    self.hierarchy.clone()
  }
}

#[cfg(test)]
mod tests {
  use std::collections::{HashMap, HashSet};

  use super::*;
  use crate::utils::*;

  #[test]
  fn test_validation() {
    let optimizer = HierarchyOptimizer::new(vec![vec![0, 1, 2]], vec![], vec![]);
    match optimizer {
      Err(OptimizerError::HierarchyMismatch { hierarchy, layers }) => {
        assert_eq!(hierarchy, 0);
        assert_eq!(layers, 1);
      }
      Err(other) => panic!("Unexpected error: {}", other),
      Ok(_) => panic!("Expected an error"),
    }

    let optimizer = HierarchyOptimizer::new(vec![vec![0, 1, 2], vec![3, 4, 5]], vec![], vec![vec![], vec![]]);
    match optimizer {
      Err(OptimizerError::EdgeLayerMismatch { edges, layers }) => {
        assert_eq!(edges, 0);
        assert_eq!(layers, 2);
      }
      Err(other) => panic!("Unexpected error: {}", other),
      Ok(_) => panic!("Expected an error"),
    }

    let optimizer = HierarchyOptimizer::new(
      vec![vec![0, 1, 2], vec![3, 4, 5]],
      vec![vec![(0, 6, 1)]],
      vec![vec![], vec![]],
    );
    match optimizer {
      Err(OptimizerError::MissingNode { node_name, layer_index }) => {
        assert_eq!(node_name, "6".to_string());
        assert_eq!(layer_index, 1);
      }
      Err(other) => panic!("Unexpected error: {}", other),
      Ok(_) => panic!("Expected an error"),
    }

    let optimizer = HierarchyOptimizer::new(
      vec![vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]],
      vec![],
      vec![vec![vec![2, 2, 2, 2, 2], vec![5, 5]]],
    );
    match optimizer {
      Err(OptimizerError::HierarchyAlignmentError {
        layer_index,
        granularity,
        next_size,
        self_size,
      }) => {
        assert_eq!(layer_index, 0);
        assert_eq!(granularity, 1);
        assert_eq!(next_size, 6);
        assert_eq!(self_size, 5);
      }
      Err(other) => panic!("Unexpected error: {}", other),
      Ok(_) => panic!("Expected an error"),
    }

    let optimizer = HierarchyOptimizer::new(
      vec![vec![0, 1, 2], vec![3, 4, 5]],
      vec![vec![(0, 3, 1), (0, 3, 1)]],
      vec![vec![], vec![]],
    );
    match optimizer {
      Err(OptimizerError::DuplicateEdge {
        node_a,
        node_b,
        layer_index,
      }) => {
        assert_eq!(node_a, "0".to_string());
        assert_eq!(node_b, "3".to_string());
        assert_eq!(layer_index, 0);
      }
      Err(other) => panic!("Unexpected error: {}", other),
      Ok(_) => panic!("Expected an error"),
    }
  }

  fn get_clusters(hierarchy: &Hierarchy, layer_index: usize, nodes: &[Vec<i32>]) -> HashMap<usize, HashSet<i32>> {
    let mut clusters = HashMap::<usize, HashSet<i32>>::new();

    for granularity in 0..hierarchy[layer_index].len() {
      let mut group_start: usize = 0;
      for group_size in &hierarchy[layer_index][granularity] {
        let node_names: HashSet<i32> = (group_start..group_start + group_size)
          .map(|i| nodes[layer_index][i])
          .collect();
        clusters.insert(*group_size, node_names);
        group_start += group_size;
      }
    }

    clusters
  }

  #[test]
  fn test_swap_hierarchy() {
    let n = 100;

    let hierarchy: Hierarchy = vec![
      vec![],
      vec![
        vec![10, 13, 7, 3, 2, 14, 20, 15, 16],
        vec![30, 19, 35, 16],
        vec![49, 51],
      ],
      vec![],
    ];

    let (nodes, edges) = gen_multi_graph(3, n).unwrap();
    let clusters = get_clusters(&hierarchy, 1, &nodes);
    let mut optimizer = HierarchyOptimizer::new(nodes, edges, hierarchy).unwrap();
    let mut start_crossings = optimizer.count_crossings();

    for granularity in [None, Some(0_usize), Some(1_usize), Some(2_usize)] {
      let end_crossings = timeit("Optimize", || optimizer.swap_nodes(1., 200, 1, granularity)).unwrap();

      assert_eq!(
        get_clusters(&optimizer.get_hierarchy(), 1, &optimizer.get_nodes()),
        clusters
      );

      assert!(start_crossings >= end_crossings, "{start_crossings} < {end_crossings}");
      assert!(end_crossings > 0);

      let real_crossings = optimizer.count_layer_crossings(1).unwrap();
      assert_eq!(end_crossings, real_crossings);
      start_crossings = end_crossings;
    }
  }

  #[test]
  fn test_cooldown_hierarchy() {
    let n = 100;

    let hierarchy: Hierarchy = vec![
      vec![],
      vec![
        vec![10, 13, 7, 3, 2, 14, 20, 15, 16],
        vec![30, 19, 35, 16],
        vec![49, 51],
      ],
      vec![],
    ];

    let (nodes, edges) = gen_multi_graph(3, n).unwrap();
    let clusters = get_clusters(&hierarchy, 1, &nodes);
    let mut optimizer = HierarchyOptimizer::new(nodes, edges, hierarchy).unwrap();
    let mut start_crossings = optimizer.count_crossings();

    for granularity in [None, Some(0_usize), Some(1_usize), Some(2_usize)] {
      let end_crossings = timeit("Optimize", || optimizer.cooldown(1., 0.1, 5, 200, 1, granularity)).unwrap();

      assert_eq!(
        get_clusters(&optimizer.get_hierarchy(), 1, &optimizer.get_nodes()),
        clusters
      );

      assert!(start_crossings >= end_crossings, "{start_crossings} < {end_crossings}");
      println!("Improved from {} to {}", start_crossings, end_crossings);
      assert!(end_crossings > 0);

      let real_crossings = optimizer.count_layer_crossings(1).unwrap();
      assert_eq!(end_crossings, real_crossings);
      start_crossings = end_crossings;
    }
  }

  #[test]
  fn test_optimize_hierarchy() {
    let n = 100;

    let hierarchy: Hierarchy = vec![
      vec![],
      vec![
        vec![10, 13, 7, 3, 2, 14, 20, 15, 16],
        vec![30, 19, 35, 16],
        vec![49, 51],
      ],
      vec![],
    ];

    let (nodes, edges) = gen_multi_graph(3, n).unwrap();
    let clusters = get_clusters(&hierarchy, 1, &nodes);
    let mut optimizer = HierarchyOptimizer::new(nodes, edges, hierarchy).unwrap();
    let start_crossings = optimizer.count_crossings();

    let end_crossings = timeit("Optimize", || optimizer.optimize(1., 0.1, 5, 200, 20)).unwrap();

    assert_eq!(
      get_clusters(&optimizer.get_hierarchy(), 1, &optimizer.get_nodes()),
      clusters
    );

    println!("Improved from {} to {}", start_crossings, end_crossings);
    assert!(start_crossings > end_crossings);
    assert!(end_crossings > 0);
  }
}
