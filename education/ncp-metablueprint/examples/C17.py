from ncp_lab.macros import macros, require
domains = {"rack-a":8,"rack-b":8}
required = 8
def may_drain(names): return sum(n for k,n in domains.items() if k not in names) >= required
require[may_drain({"rack-a"})]
require[not may_drain({"rack-a","rack-b"})]
states = ["admitted","draining","released","maintaining","validated","admitted"]
require[states.index("released") < states.index("maintaining")]
