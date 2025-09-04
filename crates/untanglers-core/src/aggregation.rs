pub fn aggregate_pairwise_matrix(pairwise_matrix: &[f64], borders: &[usize]) -> Vec<f64> {
  let new_size = borders.len();
  let mut result: Vec<f64> = vec![0.; new_size * new_size];

  if borders.len() < 2 {
    return result;
  }

  let size = borders[borders.len() - 1] + 1;
  
  let mut group_index_i = 0;
  for i in 0..size {
    if i > borders[group_index_i] {
      group_index_i += 1;
    }

    let mut group_index_j = 0;
    for j in 0..size {
      if j > borders[group_index_j] {
        group_index_j += 1;
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
    let pairwise_matrix: Vec<f64> = vec![0., 0., 3., 0., 0., 0., 6., 0., -3., -6., 0., 0., 0., 0., 0., 0.];
    let borders = vec![1, 2, 3];
    let aggregated_matrix = aggregate_pairwise_matrix(&pairwise_matrix, &borders);
    let expected_matrix: Vec<f64> = vec![0., 9., 0., -9., 0., 0., 0., 0., 0.];
    assert_eq!(aggregated_matrix, expected_matrix);
  }
}
