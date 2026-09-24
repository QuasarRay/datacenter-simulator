from ncp_lab.macros import macros, require
release = {"host":"h2", "dpu":"d2", "firmware":"f2"}
qualified = {("h1","d1","f1"), ("h2","d2","f2")}
require[tuple(release[k] for k in ("host","dpu","firmware")) in qualified]
mixed = ("h2","d1","f2")
require[mixed not in qualified]
