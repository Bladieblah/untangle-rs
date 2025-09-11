use std::fmt::{Debug, Display};
use std::hash::Hash;

use itertools::Itertools;

pub fn reorder_node_groups<T>(nodes: &[T], group_sizes: &[usize], new_indices: &[usize]) -> Vec<T>
where
  T: Eq + Hash + Clone + Display + Debug + Copy,
{
  let mut new_nodes = Vec::<T>::with_capacity(nodes.len());

  for group_index in new_indices {
    let group_start = group_sizes[0..*group_index].iter().sum();
    let group_size = group_sizes[*group_index];
    new_nodes.extend_from_slice(&nodes[group_start..group_start + group_size]);
  }

  new_nodes
}

pub fn reorder_group(parent_groups: &[usize], groups: &[usize], new_order: &[usize]) -> Vec<usize> {
  let mut new_groups = Vec::<usize>::with_capacity(groups.len());

  for parent_index in new_order {
    let parent_start: usize = parent_groups[0..*parent_index].iter().sum();
    let parent_end = parent_start + parent_groups[*parent_index];

    for i in 0..groups.len() {
      let group_start: usize = groups[0..i].iter().sum();
      if group_start >= parent_start && group_start < parent_end {
        new_groups.push(groups[i]);
      }
    }
  }

  new_groups
}

pub fn reorder_hierarchy(
  group_sizes_layers: &[Vec<usize>],
  layer_index: usize,
  new_order: &[usize],
) -> Vec<Vec<usize>> {
  // group_sizes_layers should be in order coarse -> fine
  let layer_count = group_sizes_layers.len();
  let mut new_group_sizes = Vec::<Vec<usize>>::with_capacity(layer_count);

  for l in 0..layer_count {
    if l < layer_index {
      new_group_sizes.push(group_sizes_layers[l].clone());
    } else if l == layer_index {
      new_group_sizes.push(new_order.iter().map(|i| group_sizes_layers[l][*i]).collect_vec());
    } else {
      new_group_sizes.push(reorder_group(
        &group_sizes_layers[layer_index],
        &group_sizes_layers[l],
        new_order,
      ));
    }
  }

  new_group_sizes
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_reorder_groups() {
    let nodes = vec!["A", "B", "C", "D", "E", "F", "G"];
    let group_sizes: Vec<usize> = vec![2, 2, 3];
    let new_order: Vec<usize> = vec![2, 1, 0];

    let new_nodes = reorder_node_groups(&nodes, &group_sizes, &new_order);
    assert_eq!(new_nodes, vec!["E", "F", "G", "C", "D", "A", "B"]);
  }

  #[test]
  fn test_reorder_group() {
    let parent_groups: Vec<usize> = vec![30, 20, 35, 15];
    let child_groups: Vec<usize> = vec![10, 13, 7, 3, 3, 14, 20, 15, 15];
    let new_order: Vec<usize> = vec![1, 3, 0, 2];
    let new_child_groups = reorder_group(&parent_groups, &child_groups, &new_order);
    assert_eq!(new_child_groups, vec![3, 3, 14, 15, 10, 13, 7, 20, 15]);
  }

  #[test]
  fn test_reorder_hierarchy() {
    let group_layers: Vec<Vec<usize>> = vec![
      vec![50, 50],
      vec![30, 20, 35, 15],
      vec![10, 13, 7, 3, 3, 14, 20, 15, 15],
    ];

    let new_order: Vec<usize> = vec![1, 0];
    let new_group_layers = reorder_hierarchy(&group_layers, 0, &new_order);
    assert_eq!(
      new_group_layers,
      vec![
        vec![50, 50],
        vec![35, 15, 30, 20],
        vec![20, 15, 15, 10, 13, 7, 3, 3, 14],
      ]
    );

    let new_order: Vec<usize> = vec![1, 3, 0, 2];
    let new_group_layers = reorder_hierarchy(&group_layers, 1, &new_order);
    assert_eq!(
      new_group_layers,
      vec![
        vec![50, 50],
        vec![20, 15, 30, 35],
        vec![3, 3, 14, 15, 10, 13, 7, 20, 15],
      ]
    );
  }
}
