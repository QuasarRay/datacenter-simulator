#!/bin/bash
# SPDX-License-Identifier: RPL-1.5
# Run only native NCCL. Slurm supplies rank placement, PMIx, and CUDA visibility.
set -euo pipefail
collective=${1:?collective required}
case "$collective" in
  all_reduce|all_gather|broadcast|reduce_scatter|reduce|alltoall|alltoallv|scatter|gather|sendrecv|hypercube) ;;
  *) exit 64 ;;
esac
export PATH=/usr/local/bin:/usr/local/cuda/bin:/usr/bin:/bin
export LD_LIBRARY_PATH=/opt/simulator/nccl/build/lib:/usr/local/lib:/opt/deepops/pmix/lib:/opt/deepops/hwloc/lib:/usr/local/cuda/lib64
export NCCL_NET=Socket NCCL_NET_PLUGIN=none NCCL_SOCKET_IFNAME='=simnccl'
export NCCL_SOCKET_FAMILY=AF_INET NCCL_IB_DISABLE=1 NCCL_P2P_DISABLE=1 NCCL_SHM_DISABLE=1
export NCCL_NVLS_ENABLE=0 NCCL_COLLNET_ENABLE=0 NCCL_MNNVL_ENABLE=0 NCCL_ALGO=Ring,Tree
export NCCL_DEBUG=INFO NCCL_DEBUG_FILE=/dev/stderr
export OMPI_MCA_pml=ob1 OMPI_MCA_btl=self,tcp
export OMPI_MCA_btl_tcp_if_include=simnccl OMPI_MCA_oob_tcp_if_include=simnccl
args=(-g 1 -t 1 -b 256 -e 1M -f 2 -n 5 -w 1 -c 1 -d double -T 60)
case "$collective" in all_reduce|reduce|reduce_scatter) args+=(-o all);; esac

# The pinned parser uses strtol for -r: -1 iterates every root.
case "$collective" in broadcast|reduce|scatter|gather) args+=(-r -1);; esac
# With no shared filesystem, rank zero writes here on its own VM. The runner
# locates that report by checking every compute guest after srun completes.
exec "/opt/simulator/nccl-tests/build/${collective}_perf" "${args[@]}" -J "/var/tmp/simulator-nccl/${collective}.json"
