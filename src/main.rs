use crate::utils::{generate_graph, timeit};
mod crossings;
mod utils;

pub fn main() {
  env_logger::init();

  let (nodes_left, nodes_right, edges) = generate_graph(1000);
  let mut crossings = crossings::Crossings::new(nodes_left, nodes_right, edges);
  log::info!("Start: {} edge crossings", crossings.count_crossings());
  timeit("Crossings Benchmark 1e3", || crossings.swap_nodes(1000, 10.));
  log::info!("1e3: {} edge crossings", crossings.count_crossings());
  timeit("Crossings Benchmark 1e4", || crossings.swap_nodes(10000, 1.));
  log::info!("1e4: {} edge crossings", crossings.count_crossings());
  timeit("Crossings Benchmark 1e5", || crossings.swap_nodes(100000, 0.1));
  log::info!("1e5: {} edge crossings", crossings.count_crossings());

  crossings.get_nodes();
}
