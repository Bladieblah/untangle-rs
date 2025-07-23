import random
from venv import logger

from .crossings import Crossings
import untanglers
from .utils import timeit


class Benchmark:
  def __init__(self, max_iterations, temperature) -> None:
    self.max_iterations = max_iterations
    self.temperature = temperature

    graph = self.generate_graph(1000)
    self.check_python(*graph)
    self.check_rust(*graph)

  def generate_graph(self, size: int):
    nodes_left = [str(i) for i in range(size)]
    nodes_right = [str(i) for i in range(size)]

    edges = []

    l, r = 0, 0
    k = 4

    while l < size - k - 2 and r < size - k - 2:
      dl = random.randint(1, k) # randint is inclusive
      for i in range(dl):
        edges.append((str(l), str(r + i + 1), 1))
      l += dl

      dr = random.randint(1, k)
      for i in range(dr):
        edges.append((str(l + i + 1), str(r), 1))
      r += dr
    
    from .crossings import Crossings
    c = Crossings(nodes_left, nodes_right, edges)
    logger.info(f"Sanity check: {c.count_crossings()} crossings")

    random.shuffle(nodes_left)
    random.shuffle(nodes_right)

    return nodes_left, nodes_right, edges

  @timeit
  def check_python(self, nodes_left, nodes_right, edges):
    crossings = Crossings(nodes_left, nodes_right, edges)
    crossings.swap_neighbours(self.max_iterations)

  @timeit
  def check_rust(self, nodes_left, nodes_right, edges):
    crossings = untanglers.Crossings(nodes_left, nodes_right, edges)
    crossings.swap_nodes(self.max_iterations, self.temperature)

if __name__ == "__main__":
  Benchmark(10000, 1)
