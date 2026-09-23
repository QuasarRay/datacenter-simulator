# Queue and history hot paths

## RES-01 — Queue work grows with unrelated outstanding entries

**Medium · Confirmed and measured.** [simulator/src/fabric.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/fabric.rs#L317) clones both QPs before validation; `write` does the same. Derived `Clone` recursively copies posted receives, their MR strings and all completion records. Sending one byte can allocate and copy tens of thousands of entries, including on a request that will later fail. Resource limits bound retained entries but do not make this work constant-time.

The reproduction keeps receive depth constant, sends 512 one-byte messages, polls both completions and reposts one receive after each send:

| Posted receives | Elapsed µs |
|---:|---:|
| 1 | 681 |
| 1,024 | 19,241 |
| 8,192 | 148,459 |
| 32,768 | 593,854 |

The same two-node graph is used throughout. Source inspection attributes the depth-sensitive copying to the QP clones; the benchmark is not a hardware RDMA measurement. Draining a large queue through repeated sends can accumulate quadratic copying.

**Remedy:** copy only scalar IDs/state and the front receive descriptor, then mutate queues after validation succeeds. **Acceptance:** unchanged errors/atomicity/accounting and roughly flat per-send allocation cost as unrelated queue depth increases; benchmark rejected writes as well as successful sends.

## RES-02 — Retention trimming moves every surviving history entry

**Medium · Confirmed and measured.** [simulator/src/model.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/model.rs#L397) drains from the front of a `Vec` once it reaches `history_limit`, default 10,000. Every subsequent event shifts the retained entries. This affects long-running topology/lifecycle workloads even after node-construction accounting was optimized.

| Full history limit | 20,000 `set_schedule(None, None)` calls, µs |
|---:|---:|
| 0 | 1,272 |
| 1,000 | 37,695 |
| 10,000 | 430,270 |

**Remedy:** retain a ring buffer or `VecDeque`, preserving public chronological ordering and explicit zero-retention semantics. **Acceptance:** sustained operation cost should not grow linearly with the retained history window. Do not merely shrink the default and discard useful audit history.
