# NAT-01 — Blocking work can outlive the advertised deadline

**High · Confirmed call structure; regular-file FIFO check reproduced.**

| Path | Blocking work outside effective async preemption |
|---|---|
| [simulator/src/evidence.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/evidence.rs#L20) | `File::open` before `metadata().is_file()`, then synchronous hashing |
| [simulator/src/linux.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/linux.rs#L36) | Synchronous `Command::output()` inside namespace helpers |
| [simulator/src/nccl_backend.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/nccl_backend.rs#L147) | Executable/library hashing before timeout; synchronous counter reads inside the timed future |
| [simulator/src/vm.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/vm.rs#L33) | Synchronous QEMU/image/key/tool commands |
| [simulator/src/input.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/input.rs#L9) | File open/read is byte bounded but not time bounded |
| [simulator/src/provenance.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/provenance.rs#L5) | Synchronous Git subprocess without its own deadline |

[Tokio's timeout contract](https://docs.rs/tokio/latest/tokio/time/fn.timeout.html) cannot interrupt work that does not yield. An outer timeout is not a hard deadline for these calls. Namespace construction is already documented as preceding the NCCL execution deadline; it still needs its own bounded failure behavior for unattended consumer-host runs. RDMA setup also precedes the completion polling deadline in `rdma.rs`.

## Reproduction without a GPU

Save this temporary example as `simulator/examples/identity_probe.rs` in a disposable checkout:

```rust
fn main() {
    let path = std::env::args().nth(1).unwrap();
    println!("{:?}", datacenter_simulator::evidence::FileIdentity::read(path));
}
```

```sh
cargo build --locked --manifest-path simulator/Cargo.toml --example identity_probe
probe_dir=$(mktemp -d)
mkfifo "$probe_dir/identity"
timeout 1 simulator/target/debug/examples/identity_probe "$probe_dir/identity"
# Observed: the audit's equivalent Python-supervised process exceeded 1 s
# and was terminated; no regular-file rejection was reached.
rm "$probe_dir/identity"
rmdir "$probe_dir"
```

The FIFO has no writer, so `open()` blocks before the regular-file test. This proves the local identity-read issue; it does not prove a particular GPU helper hangs in production. Byte limits alone also do not stop an endless slow input from holding a session open.

## Remedy and acceptance

Reject unsupported input kinds without a blocking open, then validate the actual opened descriptor to handle replacement races. Use owned subprocesses with process-group deadlines and bounded output for tools. If hashing is delegated to blocking workers, bound their concurrency and retain ownership after caller cancellation: a timed-out `spawn_blocking` call alone does not cancel its work. Clearly separate acquisition/build/execution/cleanup deadlines.

Exercise FIFOs, slow reads, wedged fake `ip`/`qemu-img`/Git helpers, noisy output and future cancellation. Every case must terminate or return an explicit outstanding-cleanup handle within the declared bound, without starting further native work.
