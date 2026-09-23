# Cleanup completion and retained diagnostics

## NAT-02 — Dropping an NCCL future bypasses its explicit cleanup barrier

**High · Source-confirmed contract gap; hardware outcome unverified · Existing R001.** [simulator/src/nccl_backend.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/nccl_backend.rs#L277) awaits an inner timeout, then kills and waits for workers on an error. That block is never reached if an outer `select!`, task abort or dropped caller future cancels `collective` itself. Child handles use `kill_on_drop`, which initiates termination but provides no awaited completion to the cancelling caller.

[Tokio's Child documentation](https://docs.rs/tokio/latest/tokio/process/struct.Child.html) distinguishes requesting a kill from waiting/reaping. The current source therefore cannot uphold the comment that workers have terminated before the caller can reuse GPUs on **every** cancellation path. No permanent orphan or actual CUDA context overlap was reproduced here.

**Remedy/acceptance:** let a supervisor own worker cleanup independently of the request future, with an explicit completion barrier/device lease. Cancel during bootstrap, payload transfer and synchronization; do not allow device reuse until all owned PIDs are reaped or return a clear quarantined-resource state. Retain the existing two-second error-cleanup reporting.

## NAT-03 — VM teardown can wait indefinitely and hides failures

**Medium · Source-confirmed contract gap.** [simulator/src/vm.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/vm.rs#L607) awaits each `child.kill()` sequentially and ignores every result. [simulator/src/deepops_runtime.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/deepops_runtime.rs#L83) calls it after the deployment timeout has resolved, without a cleanup deadline. [simulator/src/vm.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/vm.rs#L344) only calls `start_kill`; it cannot establish the comment's stronger before-fabric-release termination ordering.

Thus a delayed/unreapable child can hold completion/report publication beyond the configured timeout, while a failed kill can be discarded. This is a source-established lack of a bound, not evidence that the successful CPU VM CI left guests running.

**Remedy/acceptance:** signal all guests, wait under one bounded cleanup deadline, report each unresolved PID and prevent reuse of its VFIO/namespace resources. Make teardown return a structured outcome and preserve the original failure plus cleanup failures. Inject delayed waits and failed kills; test boot-future cancellation as well as post-boot shutdown.

## NAT-04 — QEMU log drains lose their completion/error channel

**Medium · Risk.** [simulator/src/bounded_log.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/bounded_log.rs#L20) calls `capture` and discards the `Completion`. [simulator/src/vm.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/vm.rs#L481) uses it for QEMU stdout/stderr. `collect` flushes output and propagates write errors through that discarded channel, so the VM owner neither waits for EOF nor observes drain failure before final evidence is considered ready. Other callers explicitly invoke `finish`; QEMU does not.

A prefix limit exists and is useful. It does not prove that retained logs are complete/durable when the run is reported. Disk-full or delayed-drain behavior was not executed against QEMU here. Unlike the patched IB drain, this collector also exits on a write error instead of continuing to consume.

**Remedy/acceptance:** retain drain handles alongside guests, finish them after child exit under a bounded deadline and record truncation/I/O failure. Continue draining after a storage failure where needed to avoid changing child behavior via broken pipes. Exercise a small synthetic output producer and failing writer; this does not require GPU hardware.
