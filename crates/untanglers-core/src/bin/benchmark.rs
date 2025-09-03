use untanglers_core::utils::generate_graph;

pub fn main() {
  env_logger::init();

  let (nodes_left, nodes_right, edges) = generate_graph(2000);
  log::info!(
    "Running benchmark with L = {} R = {} E = {}",
    nodes_left.len(),
    nodes_right.len(),
    edges.len()
  );

  // let mut crossings = crossings::Crossings::new(nodes_left, nodes_right, edges);
  // let pair_crossings = timeit("Pair crossings", || {
  //   pai(crossings::Side::Left)
  // });
  // log::info!(
  //   "Start: {} edge crossings",
  //   timeit("Count crossings", || crossings.count_crossings())
  // );
  // timeit("Crossings Benchmark 1e3", || {
  //   crossings._swap_nodes(1000, 10., &pair_crossings, crossings::Side::Left)
  // });
  // log::info!("1e3: {} edge crossings", crossings.count_crossings());
  // timeit("Crossings Benchmark 1e4", || {
  //   crossings._swap_nodes(10000, 1., &pair_crossings, crossings::Side::Left)
  // });
  // log::info!("1e4: {} edge crossings", crossings.count_crossings());
  // timeit("Crossings Benchmark 1e5", || {
  //   crossings._swap_nodes(100000, 0.1, &pair_crossings, crossings::Side::Left)
  // });
  // log::info!("1e5: {} edge crossings", crossings.count_crossings());

  // crossings.get_nodes();
}
