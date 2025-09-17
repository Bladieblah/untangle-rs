use itertools::Itertools;
use untanglers_core::{
  count_crossings::count_crossings,
  mapping::map_edges,
  pairwise::get_pairwise_matrix,
  reducer::swap_nodes,
  utils::{generate_bipartite_graph, timeit},
};

pub fn main() {
  env_logger::init();

  let (nodes_left, nodes_right, edges) = generate_bipartite_graph(2000);
  log::info!(
    "Running benchmark with L = {} R = {} E = {}",
    nodes_left.len(),
    nodes_right.len(),
    edges.len()
  );

  let mapped_edges = map_edges(&nodes_left, &nodes_right, &edges);
  let pairwise_matrix = timeit("Pair crossings", || {
    get_pairwise_matrix(nodes_left.len(), nodes_right.len(), &mapped_edges)
  });
  let mut nodes = (0..nodes_left.len()).collect_vec();
  let mut crossing_count = timeit("Count crossings", || count_crossings(&nodes_left, &nodes_right, &edges)) as i64;
  log::info!("Start: {} edge crossings", crossing_count);

  (nodes, crossing_count) = timeit("Crossings Benchmark 1e3", || {
    swap_nodes(
      nodes_left.len(),
      &pairwise_matrix,
      1000,
      10.,
      crossing_count,
      nodes,
      &None,
    )
  });
  log::info!("1e3: {} edge crossings", crossing_count);

  (nodes, crossing_count) = timeit("Crossings Benchmark 1e4", || {
    swap_nodes(
      nodes_left.len(),
      &pairwise_matrix,
      10000,
      1.,
      crossing_count,
      nodes,
      &None,
    )
  });
  log::info!("1e4: {} edge crossings", crossing_count);

  (_, crossing_count) = timeit("Crossings Benchmark 1e5", || {
    swap_nodes(
      nodes_left.len(),
      &pairwise_matrix,
      100000,
      0.1,
      crossing_count,
      nodes,
      &None,
    )
  });
  log::info!("1e5: {} edge crossings", crossing_count);
}
