import untanglers


class TestUntangleRs:
  def test_crossings_simple(self):
    nodes_left = ["a", "b", "c"]
    nodes_right = ["d", "e", "f"]
    edges = [
      ("a", "f", 1),
      ("b", "f", 1),
      ("c", "e", 1),
    ]

    rs_crossings = untanglers.Crossings(nodes_left, nodes_right, edges)
    assert rs_crossings.count_crossings() == 2
    rs_crossings.swap_nodes(10, 0)
    assert rs_crossings.count_crossings() == 0


if __name__ == "__main__":
  TestUntangleRs().test_crossings_simple()
