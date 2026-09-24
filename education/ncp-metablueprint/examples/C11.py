from ncp_lab.macros import macros, require
ownership = {"gpu0":"inference", "gpu1":"training"}
slurm = {"gpu1"}
rusternetes = {"gpu0"}
require[slurm.isdisjoint(rusternetes)]
require[slurm | rusternetes == set(ownership)]
running = {"gpu0", "gpu1"}
require["gpu1" in running]  # A requested preemption has not yet freed it.
