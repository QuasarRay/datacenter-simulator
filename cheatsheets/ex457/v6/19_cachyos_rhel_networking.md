# 19 — CachyOS host and RHEL networking

## CachyOS baseline

CachyOS documents UFW as enabled after installation and fish as the default login shell. Start `bash` for this pack's shell examples; inspect the actual firewall before troubleshooting container reachability:

```bash
bash
ip -br address
ip route
sudo ufw status verbose
systemctl is-active NetworkManager
nmcli device status
containerlab version
docker version
```

The `ip`, `nmcli` and runtime inspection sequence is this pack's diagnostic procedure, not a quoted CachyOS-certified Containerlab setup. Do not disable the firewall wholesale. Limit any necessary forwarding rule to the management subnet and actual ingress/egress interfaces, then retest and record it. CachyOS's QEMU guide demonstrates a routed UFW rule for libvirt's `192.168.122.0/24`; that example is not automatically the right rule for a Docker bridge.

If running RHEL in a CachyOS VM, follow CachyOS's QEMU/VMM prerequisites and make the intended libvirt network autostart. A NAT VM does not automatically expose its internal container management subnet to remote Controller execution nodes. Provide routed access or run the execution node in the reachable lab network.

[CachyOS post-install](https://wiki.cachyos.org/configuration/post_install_setup/). [CachyOS QEMU/VMM](https://wiki.cachyos.org/virtualization/qemu_and_vmm_setup/). [CachyOS wiki scope](https://wiki.cachyos.org/cachyos_basic/navigation-guide/).

## RHEL address and routes

Use a RHEL 9 lab VM with an existing management connection and a **spare** NIC named `ens4` (replace with the actual spare NIC). The following creates a separate study profile without a default route:

```bash
nmcli device status
sudo nmcli connection add type ethernet con-name ex457-lab ifname ens4   ipv4.method manual ipv4.addresses 192.0.2.10/24   ipv4.never-default yes ipv6.method disabled
sudo nmcli connection up ex457-lab
nmcli -f GENERAL,IP4 device show ens4
```

When `192.0.2.1` is your lab router **and actually routes** the management subnet:

```bash
sudo nmcli connection modify ex457-lab +ipv4.routes '172.30.0.0/24 192.0.2.1'
sudo nmcli connection up ex457-lab
ip route get 172.30.0.11
```

Acceptance: verify the expected device, source and gateway, reboot the VM, and inspect the persistent profile and route again. `ip route add` alone changes kernel state; the `nmcli` profile is the persistent declaration. After recording the post-reboot result, remove the practice route:

```bash
sudo nmcli connection modify ex457-lab -ipv4.routes '172.30.0.0/24 192.0.2.1'
sudo nmcli connection up ex457-lab
```

The documentation addresses RHEL behavior; the sample addressing and its applicability to this topology are this pack's explicit adaptation.

[RHEL Ethernet connections](https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/9/html/configuring_and_managing_networking/configuring-an-ethernet-connection_configuring-and-managing-networking). [RHEL static routes](https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/9/html/configuring_and_managing_networking/configuring-static-routes_configuring-and-managing-networking).

## RHEL bridge exercise

On the spare-interface lab only, first remove the `ex457-lab` profile from the previous exercise. Create a bridge and attach the unused NIC:

```bash
sudo nmcli connection delete ex457-lab
sudo nmcli connection add type bridge con-name ex457-br ifname br-ex457   ipv4.method disabled ipv6.method disabled
sudo nmcli connection add type ethernet con-name ex457-port ifname ens4   master br-ex457 slave-type bridge
sudo nmcli connection up ex457-br
sudo nmcli connection up ex457-port
bridge link show
nmcli connection show --active
```

The compatibility `master`/`slave-type` syntax is documented for RHEL 9; newer NetworkManager uses controller/port terminology. Put L3 addressing on the bridge when required, not on an enslaved port. This exercise is separate from Containerlab's automatically managed Docker bridge. Cleanup: delete `ex457-port`, then `ex457-br`; restore the spare NIC's original profile if needed. Never repurpose the NIC carrying your SSH session.

[RHEL network bridges](https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/9/html/configuring_and_managing_networking/configuring-a-network-bridge_configuring-and-managing-networking).


## Docker forwarding diagnostics

Record Docker's version and effective daemon configuration before applying a host firewall rule. Inspect `sysctl net.ipv4.ip_forward`, `sudo iptables -S FORWARD`, `sudo iptables -S DOCKER-USER` on an iptables-backed daemon, and `sudo nft list ruleset` when the installed backend uses nftables. An absent DOCKER-USER chain is not by itself a fault on the nftables backend. Compare the route from the execution node to the container and the return route; both are required.

Docker creates bridge firewall rules independently of UFW; a UFW status listing alone cannot establish container isolation or reachability. Keep Docker-managed rules intact. Do not copy a libvirt forwarding rule into a Docker topology without checking the actual interfaces/subnets. Enabling Docker's nftables backend has different forwarding prerequisites and depends on the installed Docker version. Record the selected backend and narrow any operator-managed rules to the lab path.

[Docker packet filtering and firewall behavior](https://docs.docker.com/engine/network/packet-filtering-firewalls/).
