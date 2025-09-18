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

    edges: list[tuple[str, str, int]] = []

    left, right = 0, 0
    k = 4

    while left < size - k - 4 and right < size - k - 4:
      dl = random.randint(1, k)  # randint is inclusive
      for i in range(dl):
        edges.append((str(left), str(right + i + 1), 1))
      left += dl

      dr = random.randint(1, k)
      for i in range(dr):
        edges.append((str(left + i + 1), str(right), 1))
      right += dr

    from .crossings import Crossings

    c = Crossings(nodes_left, nodes_right, edges)
    logger.info(f"Sanity check: {c.count_crossings()} crossings")

    random.shuffle(nodes_left)
    random.shuffle(nodes_right)

    return nodes_left, nodes_right, edges

  @timeit
  def check_python(self, nodes_left: list[str], nodes_right: list[str], edges: list[tuple[str, str, int]]):
    crossings = Crossings(nodes_left, nodes_right, edges)
    crossings.swap_neighbours(self.max_iterations)

  @timeit
  def check_rust(self, nodes_left: list[str], nodes_right: list[str], edges: list[tuple[str, str, int]]):
    crossings = untanglers.LayoutOptimizerString([nodes_left, nodes_right], [edges])
    crossings.swap_nodes(0, self.max_iterations, self.temperature)


if __name__ == "__main__":
  Benchmark(10000, 1)
