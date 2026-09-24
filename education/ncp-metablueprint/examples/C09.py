from ncp_lab.macros import macros, require
nodes = [{"name":"w0","synthetic":True,"advertised_gpu":8},
         {"name":"incus-compute-a","synthetic":False,"observed_gpu":1}]
physical = sum(n.get("observed_gpu",0) for n in nodes if not n["synthetic"])
require[physical == 1]
require[physical != sum(n.get("advertised_gpu",0) for n in nodes)]
