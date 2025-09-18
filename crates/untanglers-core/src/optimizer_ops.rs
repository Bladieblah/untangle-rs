use std::fmt::{Debug, Display};
use std::hash::Hash;

pub trait OptimizerOps<T>
where
  T: Eq + Hash + Clone + Display + Debug,
{
  fn count_layer_crossings(&self, layer_index: usize) -> Result<usize, OptimizerError>;
  fn count_crossings(&self) -> usize;
  fn get_nodes(&self) -> Vec<Vec<T>>;
}

pub trait OptimizerInternalOps<T>
where
  T: Eq + Hash + Clone + Display + Debug,
{
  #[allow(clippy::type_complexity)]
  fn get_adjacent_layers(
    &self,
    layer_index: usize,
  ) -> Result<(&[T], &[(T, T, usize)], Option<&Vec<T>>, Option<&Vec<(T, T, usize)>>), OptimizerError>;
}

macro_rules! impl_optimizer_ops {
  ($className:ty) => {
    impl<T> OptimizerOps<T> for $className
    where
      T: Eq + Hash + Clone + Display + Debug,
    {
      fn count_layer_crossings(&self, layer_index: usize) -> Result<usize, OptimizerError> {
        self.optimizer.count_layer_crossings(layer_index)
      }
      fn count_crossings(&self) -> usize {
        self.optimizer.count_crossings()
      }
      fn get_nodes(&self) -> Vec<Vec<T>> {
        self.optimizer.get_nodes()
      }
    }

    impl<T> OptimizerInternalOps<T> for $className
    where
      T: Eq + Hash + Clone + Display + Debug,
    {
      fn get_adjacent_layers(
        &self,
        layer_index: usize,
      ) -> Result<(&[T], &[(T, T, usize)], Option<&Vec<T>>, Option<&Vec<(T, T, usize)>>), OptimizerError> {
        self.optimizer.get_adjacent_layers(layer_index)
      }
    }
  };
}

pub(crate) use impl_optimizer_ops;

use crate::error::OptimizerError;
