from ncp_lab.macros import macros, require
permissions = {"observer": {"read"}, "provisioner": {"read", "provision"}}
require["provision" not in permissions["observer"]]
epoch, lease_epoch, fenced = 8, 8, True
successor_may_write = fenced and lease_epoch == epoch
require[successor_may_write]
require[not (fenced and 7 == epoch)]
