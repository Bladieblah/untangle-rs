use std::fmt::{Debug, Display};
use std::hash::Hash;

use itertools::Itertools;

use crate::error::OptimizerError;

pub fn reorder_node_groups<T>(nodes: &[T], group_sizes: &[usize], new_indices: &[usize]) -> Vec<T>
where
  T: Eq + Hash + Clone + Display + Debug,
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

pub fn reorder_hierarchy(hierarchy: &[Vec<usize>], granularity: usize, new_order: &[usize]) -> Vec<Vec<usize>> {
  // group_sizes_layers should be in order fine -> coarse
  let layer_count = hierarchy.len();
  let mut new_hierarchy = Vec::<Vec<usize>>::with_capacity(layer_count);

  for l in 0..layer_count {
    if l > granularity {
      new_hierarchy.push(hierarchy[l].clone());
    } else if l == granularity {
      new_hierarchy.push(new_order.iter().map(|i| hierarchy[l][*i]).collect_vec());
    } else {
      new_hierarchy.push(reorder_group(&hierarchy[granularity], &hierarchy[l], new_order));
    }
  }

  new_hierarchy
}

pub fn get_borders(child_groups: &[usize], parent_groups: &[usize]) -> Vec<usize> {
  let mut borders = Vec::<usize>::with_capacity(parent_groups.len());

  let mut parent_size: usize = 0;
  let mut child_size: usize = 0;
  let mut child_index: usize = 0;
  for group_size in parent_groups {
    parent_size += group_size;
    loop {
      child_size += child_groups[child_index];
      child_index += 1;
      if child_size == parent_size {
        borders.push(child_index - 1);
        break;
      }

      if child_size > parent_size {
        panic!("Layers out of sync, did you validate them?");
      }
    }
  }

  borders
}

/// Determines the appropriate groups and borders for swapping nodes at a given granularity.
///
/// Groups are sets of nodes that can be swapped as a whole, borders are boundaries across which nodes cannot be swapped.
/// In a hierarchy, each coarser level acts as borders for the level below it.
///
/// * `hierarchy` The group sizes for the nodes
/// * `granularity` If None, nodes aren't grouped, only borders will be given
pub fn groups_and_borders(
  hierarchy: &[Vec<usize>],
  granularity: Option<usize>,
) -> (Option<Vec<usize>>, Option<Vec<usize>>) {
  match granularity {
    None => (
      None,
      if hierarchy.is_empty() {
        None
      } else {
        Some(
          hierarchy[0]
            .iter()
            .scan(0, |state, &x| {
              *state += x;
              Some(*state - 1)
            })
            .collect_vec(),
        )
      },
    ),
    Some(granularity) => (
      Some(hierarchy[granularity].clone()),
      if granularity + 1 < hierarchy.len() {
        Some(get_borders(&hierarchy[granularity], &hierarchy[granularity + 1]))
      } else {
        None
      },
    ),
  }
}

pub fn validate_hierarchy(
  layer_index: usize,
  node_count: usize,
  hierarchy: &[Vec<usize>],
) -> Result<(), OptimizerError> {
  if hierarchy.is_empty() {
    return Ok(());
  }

  for granularity in 0..hierarchy.len() {
    let size: usize = hierarchy[granularity].iter().sum();

    for &group_size in &hierarchy[granularity] {
      if group_size == 0 {
        return Err(OptimizerError::EmptyGroup {
          layer_index,
          granularity,
        });
      }
    }

    if size != node_count {
      return Err(OptimizerError::HierarchySizeMismatch {
        layer_index,
        granularity,
        size,
        node_count,
      });
    }

    if granularity == 0 {
      continue;
    }

    let mut self_size: usize = 0;
    let mut next_size: usize = 0;
    let mut next_index: usize = 0;
    for group_size in &hierarchy[granularity] {
      self_size += group_size;
      loop {
        next_size += hierarchy[granularity - 1][next_index];
        next_index += 1;
        if next_size == self_size {
          break;
        }

        if next_size > self_size {
          return Err(OptimizerError::HierarchyAlignmentError {
            layer_index,
            granularity,
            next_size,
            self_size,
          });
        }
      }
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_reorder_nodes_by_group() {
    let nodes = vec!["A", "B", "C", "D", "E", "F", "G"];
    let group_sizes: Vec<usize> = vec![2, 2, 3];
    let new_order: Vec<usize> = vec![2, 1, 0];

    let new_nodes = reorder_node_groups(&nodes, &group_sizes, &new_order);
    assert_eq!(new_nodes, vec!["E", "F", "G", "C", "D", "A", "B"]);
  }

  #[test]
  fn test_reorder_group_sizes() {
    let parent_groups: Vec<usize> = vec![30, 20, 35, 15];
    let child_groups: Vec<usize> = vec![10, 13, 7, 3, 3, 14, 20, 15, 15];
    let new_order: Vec<usize> = vec![1, 3, 0, 2];
    let new_child_groups = reorder_group(&parent_groups, &child_groups, &new_order);
    assert_eq!(new_child_groups, vec![3, 3, 14, 15, 10, 13, 7, 20, 15]);
  }

  #[test]
  fn test_get_borders() {
    let groups1: Vec<usize> = vec![50, 50];
    let groups2: Vec<usize> = vec![30, 20, 35, 15];
    let groups3: Vec<usize> = vec![10, 13, 7, 3, 3, 14, 20, 15, 15];

    let borders1: Vec<usize> = vec![1, 3];
    let borders2: Vec<usize> = vec![5, 8];
    let borders3: Vec<usize> = vec![2, 5, 7, 8];

    assert_eq!(get_borders(&groups2, &groups1), borders1);
    assert_eq!(get_borders(&groups3, &groups1), borders2);
    assert_eq!(get_borders(&groups3, &groups2), borders3);
  }

  #[test]
  fn test_reorder_hierarchy() {
    let group_layers: Vec<Vec<usize>> = vec![
      vec![10, 13, 7, 3, 3, 14, 20, 15, 15],
      vec![30, 20, 35, 15],
      vec![50, 50],
    ];

    assert!(validate_hierarchy(0, 100, &group_layers).is_ok());

    let new_order: Vec<usize> = vec![1, 0];
    let new_group_layers = reorder_hierarchy(&group_layers, 2, &new_order);
    assert_eq!(
      new_group_layers,
      vec![
        vec![20, 15, 15, 10, 13, 7, 3, 3, 14],
        vec![35, 15, 30, 20],
        vec![50, 50],
      ]
    );

    let new_order: Vec<usize> = vec![1, 3, 0, 2];
    let new_group_layers = reorder_hierarchy(&group_layers, 1, &new_order);
    assert_eq!(
      new_group_layers,
      vec![
        vec![3, 3, 14, 15, 10, 13, 7, 20, 15],
        vec![20, 15, 30, 35],
        vec![50, 50],
      ]
    );
  }

  #[test]
  fn test_groups_and_borders() {
    let hierarchy: Vec<Vec<usize>> = vec![
      vec![10, 13, 7, 3, 3, 14, 20, 15, 15],
      vec![30, 20, 35, 15],
      vec![50, 50],
    ];

    let (groups, borders) = groups_and_borders(&[], None);
    assert_eq!(groups, None);
    assert_eq!(borders, None);

    let (groups, borders) = groups_and_borders(&hierarchy, None);
    assert_eq!(groups, None);
    assert_eq!(borders, Some(vec![9, 22, 29, 32, 35, 49, 69, 84, 99]));

    let (groups, borders) = groups_and_borders(&hierarchy, Some(0));
    assert_eq!(groups, Some(hierarchy[0].clone()));
    assert_eq!(borders, Some(vec![2, 5, 7, 8]));

    let (groups, borders) = groups_and_borders(&hierarchy, Some(1));
    assert_eq!(groups, Some(hierarchy[1].clone()));
    assert_eq!(borders, Some(vec![1, 3]));

    let (groups, borders) = groups_and_borders(&hierarchy, Some(2));
    assert_eq!(groups, Some(hierarchy[2].clone()));
    assert_eq!(borders, None);
  }
}
