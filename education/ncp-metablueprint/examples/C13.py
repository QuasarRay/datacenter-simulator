import hashlib
from ncp_lab import RouteBudget
from ncp_lab.macros import macros, require
checkpoint = b"epoch=12;weights=example"
expected = hashlib.sha256(checkpoint).hexdigest()
require[hashlib.sha256(checkpoint[:-1]).hexdigest() != expected]
budget = RouteBudget(100_000_000_000, 1000)
require[budget.optimistic_ns(1_000_000) == 81_000]
# Cache, queueing, storage and GPU time are additional measured costs.
