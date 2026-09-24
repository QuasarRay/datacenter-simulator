from ncp_lab import Envelope, commissioning
from ncp_lab.macros import macros, require
plan = commissioning(Envelope(gpus=8, watts=4000, rack_watts=5000, cooling_watts=4500))
require[plan.admitted]
try:
    commissioning(Envelope(8, 4000, 5000, 3500))
except ValueError:
    rejected = True
else:
    rejected = False
require[rejected]
