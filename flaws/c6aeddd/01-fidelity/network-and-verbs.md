# Network and verbs semantics

## FID-03 — Whole-message timing differs from packet forwarding

**High · Gap · Existing G002.** [simulator/src/topology.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/topology.rs#L49) chooses one minimum-latency path with hop-count/tie rules, independent of message size and bandwidth. [simulator/src/topology.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/topology.rs#L271) serializes the entire message at every hop with lossless queues. The batch must finish before another overlapping batch can be submitted. Reset explicitly cancels all scheduled traffic, even when resetting an unrelated node.

In contrast, [simulator/src/linux.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/linux.rs#L187) uses real TCP, 1500-byte interfaces and finite `netem` queues; [simulator/src/model.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/model.rs#L166) sizes queues from two bandwidth-delay products plus 1,024 packets. There is no shared packet/flow model, ECN/PFC, IB credits, packet loss/retransmission model, ECMP or routing-convergence process. [Cumulus ECMP](https://docs.nvidia.com/networking-ethernet-software/cumulus-linux-59/Layer-3/Routing/Equal-Cost-Multipath-Load-Sharing/) uses multiple equal-cost next hops, which the one-path graph cannot reproduce.

**Concrete consequence:** on two 100-Gb/s hops with 1 µs propagation each, a 1-MiB analytical message completes at `2 × (83,887 + 1,000) = 169,774 ns`, ignoring contention. A packetized pipeline does not wait for the complete 1 MiB at each intermediate switch. This arithmetic is an illustration of the model, not a measured NVIDIA latency. Low-latency routing can also select a much slower bandwidth path.

**Remedy and acceptance:** define whether a mode predicts messages, flows or packets. Add bounded streaming events, finite buffers and explicit congestion behavior where predictions need them; use analytical flow models for scale rather than one heap object per byte/packet. Test incast, asymmetric rates, multipath, loss, timed faults and unrelated-node resets against independent reference traces. Preserve the corrected downstream FIFO and cable-orientation regressions from PR #12.

## FID-04 — Software RC is a memory/queue contract, not RNIC behavior

**Medium · Gap with local probe · Existing G006.** [simulator/src/fabric.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/fabric.rs#L25) only exposes Reset/Rts/Error. `send` rejects an empty receive queue immediately, leaves the QP in Rts, and creates no error completion. Completion records contain no status. Protection is associated with a node rather than separately allocated protection domains; RDMA read, atomics, retry timers, flush completions and asynchronous CQ events are absent.

A native-independent public-API probe observed:

```text
send without posted receive -> Err(State("receiver not ready (RNR)"))
QP state -> Ok(Rts)
poll -> Ok(None)
```

NVIDIA's [RDMA programming manual](https://docs.nvidia.com/rdma-aware-networks-programming-user-manual-1-7.pdf), sections 4.5 and 7, specifies staged QP bring-up and retry/error completion behavior. The immediate rejection here must not be used to predict RC recovery or failure timing. [simulator/src/rdma.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/rdma.rs#L28) is a separate real-device loopback and does not close this modeling gap.

**Remedy and acceptance:** either keep the API explicitly scoped to synchronous contract checking, or add a bounded work-request state machine with independent PD/CQ identities and statuses. Validate RNR exhaustion, short receives, bad keys, peer destruction and flushed work against a configured RNIC/SoftRoCE fixture. Do not call the software model a hardware fallback.
