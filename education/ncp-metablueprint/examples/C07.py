from ncp_lab import pkey_allows
from ncp_lab.macros import macros, require
require[pkey_allows(0x8001, 0x0001)]
require[not pkey_allows(0x0001, 0x0001)]
require[not pkey_allows(0x8001, 0x8002)]
