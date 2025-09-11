pub fn aggregate_pairwise_matrix(pairwise_matrix: &[f64], group_sizes: &[usize]) -> Vec<f64> {
  let new_size = group_sizes.len();
  let mut result: Vec<f64> = vec![0.; new_size * new_size];

  if group_sizes.len() < 2 {
    return result;
  }

  let size = group_sizes.iter().sum();

  let mut group_index_i = 0;
  let mut group_end_i = group_sizes[0];
  for i in 0..size {
    if i >= group_end_i {
      group_index_i += 1;
      group_end_i += group_sizes[group_index_i];
    }

    let mut group_index_j = 0;
    let mut group_end_j = group_sizes[0];
    for j in 0..size {
      if j >= group_end_j {
        group_index_j += 1;
        group_end_j += group_sizes[group_index_j];
      }

      // Due to the antisymmetry diagonals remain 0 after aggregation
      if group_index_i == group_index_j {
        continue;
      }

      result[group_index_j * new_size + group_index_i] += pairwise_matrix[j * size + i];
    }
  }

  result
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_aggregation() {
    // 4 by 4 matrix, same as in reducer::test_simple_graph
    let pairwise_matrix: Vec<f64> = vec![0., 0., 3., 0., 0., 0., 6., 0., -3., -6., 0., 0., 0., 0., 0., 0.];
    let borders = vec![2, 1, 1];
    let aggregated_matrix = aggregate_pairwise_matrix(&pairwise_matrix, &borders);
    let expected_matrix: Vec<f64> = vec![0., 9., 0., -9., 0., 0., 0., 0., 0.];
    assert_eq!(aggregated_matrix, expected_matrix);
  }
}
