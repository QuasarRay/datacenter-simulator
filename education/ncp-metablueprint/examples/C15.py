from ncp_lab.macros import macros, require
samples = [{"second":i,"errors":0,"participants":4,"progress":10+i} for i in range(5)]
require[all(s["participants"] == 4 and s["errors"] == 0 for s in samples)]
require[all(a["progress"] < b["progress"] for a,b in zip(samples,samples[1:]))]
require[samples[-1]["second"]-samples[0]["second"] == 4]
# Five model samples are not a hardware soak.
