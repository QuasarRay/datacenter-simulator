from ncp_lab.macros import macros, require
requests = {"team-a": 2, "team-b": 2}
allocations = {"team-a": 2, "team-b": 2}
physical = 4
require[sum(allocations.values()) <= physical]
require[all(0 < n <= allocations[t] for t,n in requests.items())]
drained = {"gpu-3"}
placement = {"gpu-0", "gpu-1"}
require[placement.isdisjoint(drained)]
