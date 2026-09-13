# 20 — Runtime release gate

## Required live artifacts

All rows start **NOT RUN** in this release. Save real outputs under `.state/runtime/`, remove secrets, and attach them to a follow-up review. A document review is not a substitute for any row.

| Gate | Action | Pass evidence |
|---|---|---|
| R1 image/account | Build inspected digest; inspect account and effective sshd policy | Build log, FRR version, unlocked account, password auth disabled |
| R2 SSH transport | Manual login; network_cli show-version; replace trust with wrong key | Correct-key success and wrong-key failure in host and EE modes |
| R3 convergence | Configure twice; full verification | Exact peers/config plus converged second run |
| R4 negative state | Extra Idle peer, wrong ASN, swapped address, removed route | Each fault fails before repair |
| R5 data plane | labctl dataplane; controlled total-loss test | All 12 paths pass normally; 100%-loss case fails |
| R6 saved/restart | Save; inspect startup; lifecycle restart; verify without reapply | Matching saved and post-restart state |
| R7 recovery | Backup; drift; restore actual backup; full verify | Backup file consumed; recovered state passes |
| R8 EE distribution | Entitled base pull, build, push and execution-node pull | Image digests and component list |
| R9 Controller | CaC twice; sync; launch; inspect nodes and trust | Object graph, convergence, core successful jobs, wrong-key failure |
| R10 durable backup | PUT, GET, checksum; recover downloaded backup | Matching remote bytes and successful recovery |
| R11 host lifecycle | Reboot RHEL exercise VM | Persistent profile/route and management reachability |

For R1 inspect with `sudo docker exec clab-ex457-v6-spine1 passwd -S ansible` and `sudo docker exec clab-ex457-v6-spine1 sshd -T`. Do not publish `/etc/shadow` or private key material. For R2 make a temporary public trust file for one host with a different key; use `EX457_KNOWN_HOSTS` only for the negative run. Restore the verified file for recovery.

For R5, on the isolated container use its firewall or temporarily remove the destination loopback, run the data-plane gate expecting failure, then restore the intended state with the configure play. No release status is upgraded merely because a log file exists: review its commands, expected negative outcome, hashes and environment.

[OpenSSH sshd](https://man.openbsd.org/sshd). [network_cli connection](https://docs.ansible.com/projects/ansible/latest/collections/ansible/netcommon/network_cli_connection.html). [FRR integrated configuration](https://docs.frrouting.org/en/stable-10.7/vtysh.html). [Containerlab restart command](https://containerlab.dev/cmd/restart/).

## Evidence vocabulary

`CLOSED_STATIC`: shipped code/docs plus an identified static or fixture check address the finding.

`RUNTIME_GATED`: implementation exists, but a live acceptance transcript is still needed.

`ACCEPTED_LIMIT`: the limitation is explicitly bounded, not represented as verified behavior.

`CLOSED_RUNTIME`: reserved for an actual reviewed live transcript. No v6 row uses this label yet.

The overall artifact is `STATIC_AND_FIXTURE_VALIDATED_RUNTIME_GATED` only if its available checks pass. Documentation coverage is independently computed as 23/23 mapped public objectives.




## v6 automated and external gates

The Dagger pipeline executes the lab/image/transport/negative/persistence gates through `tests/pyats_live.py`. Each named required gate must appear as PASS in that run's `artifacts/results.json`; any absent gate fails CI. Reports identify their run ID and runtime records include source/model digests. A prior run's successful dataplane record is replaced before a new attempt starts.

External acceptance remains required for the actual Controller workflow and credentials, entitled EE build/push/pull, remote durable storage service, CachyOS host networking and RHEL VM reboot. Default CI reports these as NOT_RUN. Follow this chapter's R1–R11 procedure and record the target versions, exact Git revision, operator, commands, return codes and raw sanitized outputs. Do not turn a checklist tick into a runtime PASS.
