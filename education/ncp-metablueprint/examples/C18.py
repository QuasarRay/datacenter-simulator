from ncp_lab.macros import macros, require
tables = {"routes":(900,1200,200), "neighbors":(400,800,100)}
require[all(used+reserve <= total for used,total,reserve in tables.values())]
locality = {"gpu0":0,"nic0":0,"cpu-worker":0}
require[len(set(locality.values())) == 1]
