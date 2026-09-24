from unpythonic import pipe1
from ncp_lab.macros import macros, require
tenants = [{"name":"red","vni":10001,"rt":"65000:1"}, {"name":"blue","vni":10002,"rt":"65000:2"}]
vnis = pipe1(tenants, lambda rows: [r["vni"] for r in rows])
require[len(vnis) == len(set(vnis))]
imports = {row["name"]: {row["rt"]} for row in tenants}
require[imports["red"].isdisjoint(imports["blue"])]
