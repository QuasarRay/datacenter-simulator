from ncp_lab.macros import macros, require
rails = [("gpu-A", "nic-A", "leaf1:1"), ("gpu-B", "nic-B", "leaf1:2")]
require[len({gpu for gpu, _, _ in rails}) == len(rails)]
require[len({port for _, _, port in rails}) == len(rails)]
observed = {"nic-A": "leaf1:1", "nic-B": "leaf1:2"}
require[all(observed[nic] == port for _, nic, port in rails)]
