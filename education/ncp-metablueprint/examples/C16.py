from ncp_lab.macros import macros, require
def overlap(a,b): return max(a[0],b[0]) <= min(a[1],b[1])
gpu = {"job":"j7","window":(100,104)}
nic = {"job":"j7","window":(103,107)}
unrelated = {"job":"j8","window":(100,101)}
require[gpu["job"] == nic["job"] and overlap(gpu["window"],nic["window"])]
require[not (gpu["job"] == unrelated["job"] and overlap(gpu["window"],unrelated["window"]))]
