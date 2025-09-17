import untanglers

opt = untanglers.LayoutOptimizer(
  [
    ["a", "b"],
    ["c", "d"],
  ],
  [[
    ("a", "c", 1),
    ("a", "e", 1),
  ]]
)
print(opt.count_crossings())
