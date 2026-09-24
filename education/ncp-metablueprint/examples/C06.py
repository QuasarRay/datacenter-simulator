from ncp_lab import headroom_bytes
from ncp_lab.macros import macros, require
minimum = headroom_bytes(400_000_000_000, 1000, 9216)
require[minimum == 59216]
require[headroom_bytes(400_000_000_000, 2000, 9216) > minimum]
# This lower bound is not a Spectrum buffer configuration.
