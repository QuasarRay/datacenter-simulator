from ncp_lab.macros import macros, require
applied = 12
pending = [9,12,13,11]
accepted = [revision for revision in pending if revision > applied]
require[accepted == [13]]
identity = ("edge-a","gpu-uuid-a","image-digest-a")
require[identity != ("edge-a","gpu-uuid-b","image-digest-a")]
