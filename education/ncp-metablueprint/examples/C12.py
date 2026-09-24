from ncp_lab import mig_fits
from ncp_lab.macros import macros, require
# Illustrative symbols only; replace with the observed model-specific catalogue.
catalogue = [("small@0","small@1"), ("large@0",)]
require[mig_fits(("small@0","small@1"), catalogue)]
require[not mig_fits(("small@1","small@0"), catalogue)]
