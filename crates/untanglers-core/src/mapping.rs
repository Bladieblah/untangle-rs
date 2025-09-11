use itertools::Itertools;
use std::collections::HashMap;
use std::hash::Hash;

pub fn swap_edges<T>(edges: &[(T, T, usize)]) -> Vec<(T, T, usize)>
where
  T: Eq + Hash + Clone,
{
  edges.iter().map(|(l, r, w)| (r.clone(), l.clone(), *w)).collect_vec()
}

pub fn invert_vec<T>(v: &[T]) -> HashMap<&T, usize>
where
  T: Eq + Hash + Clone,
{
  v.iter().enumerate().map(|(i, item)| (item, i)).collect()
}

pub fn map_edges<T>(nodes1: &[T], nodes2: &[T], edges: &[(T, T, usize)]) -> Vec<(usize, usize, usize)>
where
  T: Eq + Hash + Clone,
{
  let index1 = invert_vec(nodes1);
  let index2 = invert_vec(nodes2);

  edges.iter().map(|(l, r, w)| (index1[l], index2[r], *w)).collect_vec()
}

pub fn reorder_nodes<T>(nodes: &[T], new_indices: &[usize]) -> Vec<T>
where
  T: Eq + Hash + Clone,
{
  new_indices.iter().map(|l| nodes[*l].clone()).collect_vec()
}
