from ncp_lab.macros import macros, require
order = ["fence","identity","fabric","storage","admission","workload"]
edges = [("fence","identity"),("identity","fabric"),("fabric","admission"),
         ("storage","admission"),("admission","workload")]
position = {name:i for i,name in enumerate(order)}
require[all(position[a] < position[b] for a,b in edges)]
current_epoch, report_epoch = 31, 30
require[current_epoch != report_epoch]
