from ncp_lab.macros import macros, require
ranks = [("node-a",0),("node-b",0)]
inputs = [[1.0,2.0],[3.0,4.0]]
require[len(ranks) == len(inputs)]
require[len({tuple(x) for x in ranks}) == len(ranks)]
require[len({len(x) for x in inputs}) == 1]
# Shape validation only. Actual reduction is delegated to native NCCL.
