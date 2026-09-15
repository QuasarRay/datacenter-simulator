// SPDX-License-Identifier: RPL-1.5
// Rust simulator extension of QuasarRay/datacenter-simulator.
// Unless explicitly acquired and licensed from Licensor under another license,
// the contents of this file are subject to the Reciprocal Public License
// ("RPL") Version 1.5, or subsequent versions as allowed by the RPL, and You
// may not copy or use this file in either source code or executable form,
// except in compliance with the terms and conditions of the RPL.
// All software distributed under the RPL is provided strictly on an "AS IS"
// basis, WITHOUT WARRANTY OF ANY KIND, EITHER EXPRESS OR IMPLIED, AND LICENSOR
// HEREBY DISCLAIMS ALL SUCH WARRANTIES, INCLUDING WITHOUT LIMITATION, ANY
// WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, QUIET
// ENJOYMENT, OR NON-INFRINGEMENT. See ../license.md for the RPL's specific
// language governing rights and limitations.

//! Full Linux guests attached to the modeled kernel fabric through TAP file descriptors.
use crate::{
    deepops::{DeepOpsConfig, DeepOpsPlan, GuestPlan},
    linux::LinuxFabric,
    model::Simulation,
};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    os::fd::AsRawFd,
    path::Path,
    process::Stdio,
    time::Duration,
};
use tokio::process::{Child, Command};

pub(crate) fn checked(program: &str, args: &[&str]) -> Result<String> {
    let output = std::process::Command::new(program)
        .args(args)
        .output()
        .with_context(|| format!("start {program}"))?;
    ensure!(
        output.status.success(),
        "{program} {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8(output.stdout)?)
}
pub(crate) fn path(path: &Path) -> Result<&str> {
    let value = path.to_str().context("path must be UTF-8")?;
    ensure!(
        !value.chars().any(char::is_whitespace) && !value.contains([',', '%', '\'', '"', ':']),
        "paths must not contain whitespace or QEMU/Ansible option delimiters: {value}"
    );
    Ok(value)
}
/// Read-only host checks. GPU devices must already be dedicated and bound to VFIO.
pub fn preflight(config: &DeepOpsConfig, gpu: bool) -> Result<()> {
    for p in [
        &config.repository,
        &config.image,
        &config.state_dir,
        &config.ansible_bin,
    ] {
        path(p)?;
    }
    ensure!(
        !config.state_dir.exists(),
        "state_dir must be new; previous guest disks and evidence are retained"
    );
    for tool in [
        "qemu-system-x86_64",
        "qemu-img",
        "cloud-localds",
        "ssh",
        "ssh-keygen",
        "ip",
        "nft",
        "sha256sum",
    ] {
        checked("which", &[tool])?;
    }
    ensure!(Path::new("/dev/net/tun").exists(), "missing /dev/net/tun");
    let digest = checked("sha256sum", &[path(&config.image)?])?;
    ensure!(
        digest.split_whitespace().next() == Some(config.image_sha256.to_ascii_lowercase().as_str()),
        "cloud image SHA-256 mismatch"
    );
    if gpu {
        let mut limit = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        ensure!(
            unsafe { libc::getrlimit(libc::RLIMIT_MEMLOCK, &mut limit) } == 0,
            "read VFIO memlock limit"
        );
        ensure!(
            limit.rlim_cur == libc::RLIM_INFINITY,
            "VFIO requires an unlimited process memlock limit; launch with sudo prlimit --memlock=unlimited:unlimited"
        );
        OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/kvm")
            .context("DeepOps GPU deployment requires accessible KVM")?;
        let mut groups = BTreeSet::new();
        for name in &config.compute {
            let devices = config
                .vfio
                .get(name)
                .context("each compute VM requires a dedicated VFIO GPU")?;
            ensure!(
                !devices.is_empty(),
                "each compute VM requires a dedicated VFIO GPU"
            );
            let mut vm_groups = BTreeSet::new();
            let mut gpu_found = false;
            for bdf in devices {
                let device = Path::new("/sys/bus/pci/devices").join(bdf);
                ensure!(
                    fs::read_link(device.join("driver"))?.ends_with("vfio-pci"),
                    "{bdf} is not bound to vfio-pci; host drivers will not be changed"
                );
                let class = fs::read_to_string(device.join("class"))?;
                gpu_found |= class.starts_with("0x03")
                    && fs::read_to_string(device.join("vendor"))?.trim() == "0x10de";
                let group = fs::canonicalize(device.join("iommu_group"))?;
                for member in fs::read_dir(group.join("devices"))? {
                    let member = member?;
                    ensure!(
                        devices.contains(&member.file_name().to_string_lossy().to_string()),
                        "all functions in an IOMMU group must be assigned to the same guest"
                    );
                }
                vm_groups.insert(group);
            }
            ensure!(
                gpu_found,
                "{name} has no NVIDIA display/compute PCI function"
            );
            for group in vm_groups {
                ensure!(
                    groups.insert(group),
                    "IOMMU group assigned to multiple guests"
                );
            }
        }
    } else {
        ensure!(
            config.vfio.is_empty(),
            "VM smoke must not attach GPUs; use deepops-run for GPU deployment"
        );
    }
    Ok(())
}
impl LinuxFabric {
    fn attach_tap(&self, guest: &GuestPlan) -> Result<File> {
        let dev = self.device(&guest.node_id)?;
        let tap = dev.run_sync(|| {
            let tap = OpenOptions::new().read(true).write(true).open("/dev/net/tun")?;
            // ifreq has the kernel ABI layout; ioctl writes only within that struct.
            let mut request: libc::ifreq = unsafe { std::mem::zeroed() };
            for (to, from) in request.ifr_name.iter_mut().zip(b"vmtap0") { *to = *from as libc::c_char; }
            request.ifr_ifru.ifru_flags = (libc::IFF_TAP | libc::IFF_NO_PI) as libc::c_short;
            ensure!(unsafe { libc::ioctl(tap.as_raw_fd(), libc::TUNSETIFF, &mut request) } >= 0, "TUNSETIFF: {}", std::io::Error::last_os_error());
            checked("ip", &["address", "add", "10.254.0.1/30", "dev", "vmtap0"])?;
            checked("ip", &["link", "set", "vmtap0", "up"])?;
            checked("sysctl", &["-qw", "net.ipv4.ip_forward=1", "net.ipv4.conf.vmtap0.rp_filter=0"])?;
            // A host only forwards to/from its guest, never between two physical links.
            checked("nft", &["add table ip vm_boundary; add chain ip vm_boundary forward { type filter hook forward priority 0; policy drop; }; add rule ip vm_boundary forward iifname vmtap0 accept; add rule ip vm_boundary forward oifname vmtap0 accept"])?;
            Ok(tap)
        })?;
        Ok(tap)
    }
    pub(crate) fn route_guests(&self, guests: &[GuestPlan]) -> Result<()> {
        for node in self.simulation.nodes() {
            let dev = self.device(&node.id)?;
            crate::linux::command(
                dev,
                "ip",
                &crate::linux::args(&["route", "flush", "proto", "187"]),
            )?;
            for guest in guests {
                let (gateway, iface) = if guest.node_id == node.id {
                    ("10.254.0.2".to_string(), "vmtap0".to_string())
                } else {
                    let Ok(route) = self.simulation.route(&node.id, &guest.node_id, 0) else {
                        continue;
                    };
                    let link = self
                        .simulation
                        .links()
                        .find(|l| Some(&l.id) == route.first())
                        .context("guest route has no first link")?;
                    let a = self.simulation.interface(&link.interfaces[0])?;
                    let b = self.simulation.interface(&link.interfaces[1])?;
                    let (from, to) = if a.node == node.id { (a, b) } else { (b, a) };
                    (self.addresses[&to.id].to_string(), from.name.clone())
                };
                crate::linux::command(
                    dev,
                    "ip",
                    &crate::linux::args(&[
                        "route",
                        "replace",
                        &format!("{}/32", guest.address),
                        "via",
                        &gateway,
                        "dev",
                        &iface,
                        "proto",
                        "187",
                        "src",
                        &self.address(&node.id)?.to_string(),
                    ]),
                )?;
            }
        }
        Ok(())
    }
}

/// Kill command descendants as well as their parent when a stage times out.
pub(crate) struct ProcessGroup(pub u32);
impl Drop for ProcessGroup {
    fn drop(&mut self) {
        // The ID belongs to a child we started with process_group(0).
        unsafe {
            libc::kill(-(self.0 as i32), libc::SIGKILL);
        }
    }
}

pub struct VmLab {
    // Children are killed before the fabric is released, including on timeout/error.
    children: Vec<Child>,
    pub fabric: LinuxFabric,
    pub plan: DeepOpsPlan,
}
impl Drop for VmLab {
    fn drop(&mut self) {
        for child in &mut self.children {
            let _ = child.start_kill();
        }
    }
}
impl VmLab {
    pub(crate) async fn boot(
        config: &DeepOpsConfig,
        sim: &Simulation,
        plan: DeepOpsPlan,
    ) -> Result<Self> {
        let key = config.state_dir.join("id_ed25519");
        checked(
            "ssh-keygen",
            &["-q", "-t", "ed25519", "-N", "", "-f", path(&key)?],
        )?;
        let public = fs::read_to_string(key.with_extension("pub"))?;
        fs::write(config.state_dir.join("known_hosts"), "")?;
        let mut lab = Self {
            children: Vec::new(),
            fabric: LinuxFabric::build(sim).await?,
            plan,
        };
        let kvm = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/kvm")
            .is_ok();
        for (i, guest) in lab.plan.guests.iter().enumerate() {
            let dir = config.state_dir.join("guests").join(&guest.name);
            fs::create_dir(&dir)?;
            checked(
                "qemu-img",
                &[
                    "create",
                    "-f",
                    "qcow2",
                    "-F",
                    "qcow2",
                    "-b",
                    path(&config.image)?,
                    path(&dir.join("disk.qcow2"))?,
                    &format!("{}G", guest.disk_gib),
                ],
            )?;
            let mac = format!("52:54:00:dc:00:{:02x}", i + 1);
            let management_mac = format!("52:54:00:dc:01:{:02x}", i + 1);
            let service = String::from(
                "[Unit]\nDescription=Simulator fabric endpoint\nWants=network-online.target\nAfter=network-online.target\n[Service]\nType=oneshot\nRemainAfterExit=yes\nExecStart=/usr/local/sbin/simulator-network\n[Install]\nWantedBy=multi-user.target\n",
            );
            let setup = format!(
                "#!/bin/sh\nset -eu\nip link show simnccl >/dev/null 2>&1 || ip link add simnccl type dummy\nip address replace {}/32 dev simnccl\nip link set simnccl up\nip route replace 10.253.0.0/16 via 10.254.0.1 dev fabric0 src {}\nip route replace 10.255.0.0/16 via 10.254.0.1 dev fabric0 src {}\nsysctl -qw net.ipv4.conf.all.rp_filter=0 net.ipv4.conf.fabric0.rp_filter=0\n",
                guest.address, guest.address, guest.address
            );
            let user = json!({"hostname":guest.name,"manage_etc_hosts":true,"ssh_pwauth":false,"disable_root":false,
                "users":[{"name":"deepops","shell":"/bin/bash","sudo":"ALL=(ALL) NOPASSWD:ALL","lock_passwd":true,"ssh_authorized_keys":[public.trim()]}],
                "write_files":[{"path":"/etc/simulator-guest-id","content":guest.name},
                    {"path":"/usr/local/sbin/simulator-network","permissions":"0755","content":setup},
                    {"path":"/etc/systemd/system/simulator-network.service","content":service}],
                "runcmd":[["systemctl","daemon-reload"],["systemctl","enable","--now","simulator-network.service"]]});
            fs::write(
                dir.join("user-data"),
                format!("#cloud-config\n{}\n", serde_json::to_string_pretty(&user)?),
            )?;
            fs::write(
                dir.join("meta-data"),
                serde_json::to_vec(
                    &json!({"instance-id":uuid::Uuid::new_v4().to_string(),"local-hostname":guest.name}),
                )?,
            )?;
            let network = json!({"version":2,"ethernets":{
                "fabric0":{"match":{"macaddress":mac},"set-name":"fabric0","dhcp4":false,"addresses":["10.254.0.2/30"]},
                "management0":{"match":{"macaddress":management_mac},"set-name":"management0","dhcp4":true,"optional":true}
            }});
            fs::write(
                dir.join("network-config"),
                serde_json::to_vec_pretty(&network)?,
            )?;
            checked(
                "cloud-localds",
                &[
                    "--network-config",
                    path(&dir.join("network-config"))?,
                    path(&dir.join("seed.iso"))?,
                    path(&dir.join("user-data"))?,
                    path(&dir.join("meta-data"))?,
                ],
            )?;
            let tap = lab.fabric.attach_tap(guest)?;
            let fd = tap.as_raw_fd();
            let mut cmd = Command::new("qemu-system-x86_64");
            cmd.args([
                "-name",
                &guest.name,
                "-machine",
                if kvm {
                    "q35,accel=kvm"
                } else {
                    "q35,accel=tcg"
                },
                "-cpu",
                if kvm { "host" } else { "max" },
                "-smp",
                &guest.cpus.to_string(),
                "-m",
                &guest.memory_mib.to_string(),
                "-display",
                "none",
                "-monitor",
                "none",
                "-serial",
                &format!("file:{}", path(&dir.join("serial.log"))?),
                "-drive",
                &format!(
                    "file={},if=virtio,format=qcow2",
                    path(&dir.join("disk.qcow2"))?
                ),
                "-drive",
                &format!(
                    "file={},if=virtio,format=raw,readonly=on",
                    path(&dir.join("seed.iso"))?
                ),
                "-netdev",
                &format!("tap,id=fabric,fd={fd}"),
                "-device",
                &format!("virtio-net-pci,netdev=fabric,mac={mac}"),
                "-netdev",
                "user,id=management",
                "-device",
                &format!("virtio-net-pci,netdev=management,mac={management_mac}"),
            ]);
            for bdf in &guest.vfio {
                cmd.args(["-device", &format!("vfio-pci,host={bdf}")]);
            }
            cmd.stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(File::create(dir.join("qemu.log"))?)
                .kill_on_drop(true);
            let parent = unsafe { libc::getpid() };
            // Only async-signal-safe syscalls after fork. Inherit exactly this TAP descriptor.
            unsafe {
                cmd.pre_exec(move || {
                    let flags = libc::fcntl(fd, libc::F_GETFD);
                    if flags < 0
                        || libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) < 0
                        || libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) != 0
                    {
                        return Err(std::io::Error::last_os_error());
                    }
                    if libc::getppid() != parent {
                        return Err(std::io::Error::other("supervisor exited"));
                    }
                    Ok(())
                });
            }
            lab.children.push(cmd.spawn()?);
        }
        lab.fabric.route_guests(&lab.plan.guests)?;
        for guest in lab.plan.guests.clone() {
            eprintln!("waiting for VM {} ({})", guest.name, guest.address);
            let ready = async {
                loop {
                    if let Ok(output) = lab
                        .ssh(
                            config,
                            &guest,
                            &["sudo", "cloud-init", "status", "--wait"],
                            Duration::from_secs(15),
                        )
                        .await
                        && output.status.success()
                    {
                        return Ok::<_, anyhow::Error>(());
                    }
                    for child in &mut lab.children {
                        ensure!(
                            child.try_wait()?.is_none(),
                            "QEMU exited during boot; inspect guests/*/qemu.log"
                        );
                    }
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            };
            tokio::time::timeout(Duration::from_secs(config.boot_timeout_secs), ready)
                .await
                .context("guest boot timed out; inspect guests/*/serial.log")??;
        }
        Ok(lab)
    }
    pub(crate) fn ssh_command(
        &self,
        config: &DeepOpsConfig,
        guest: &GuestPlan,
        remote: &[&str],
    ) -> Result<Command> {
        let mut cmd = Command::new("ssh");
        cmd.args([
            "-i",
            path(&config.state_dir.join("id_ed25519"))?,
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=5",
            "-o",
            "IdentitiesOnly=yes",
            "-o",
            "StrictHostKeyChecking=accept-new",
            "-o",
            &format!(
                "UserKnownHostsFile={}",
                path(&config.state_dir.join("known_hosts"))?
            ),
            &format!("deepops@{}", guest.address),
        ])
        .args(remote)
        .kill_on_drop(true);
        Ok(cmd)
    }
    pub(crate) async fn ssh(
        &self,
        config: &DeepOpsConfig,
        guest: &GuestPlan,
        remote: &[&str],
        duration: Duration,
    ) -> Result<std::process::Output> {
        let mut cmd = self.ssh_command(config, guest, remote)?;
        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());
        cmd.process_group(0);
        let child = self
            .fabric
            .device(&self.plan.provisioner_id)?
            .spawn_command(cmd)?;
        let _group = ProcessGroup(child.id().context("SSH process has no PID")?);
        Ok(tokio::time::timeout(duration, child.wait_with_output()).await??)
    }
    pub async fn shutdown(&mut self) {
        for child in &mut self.children {
            let _ = child.kill().await;
        }
    }
    pub async fn smoke(&mut self, config: &DeepOpsConfig) -> Result<Value> {
        for guest in &self.plan.guests {
            let out = self
                .ssh(
                    config,
                    guest,
                    &["cat", "/etc/simulator-guest-id"],
                    Duration::from_secs(10),
                )
                .await?;
            ensure!(
                out.status.success() && String::from_utf8_lossy(&out.stdout).trim() == guest.name,
                "guest identity/isolation failed"
            );
        }
        let from = self.plan.guests[1].clone();
        let to = self.plan.guests[2].clone();
        // Select the routable source address. Binding ping to the dummy device
        // would force packets onto that device instead of the physical route.
        let source = from.address.to_string();
        let ping = ["ping", "-I", &source, "-n", "-c", "2", "-W", "2"];
        let ip = to.address.to_string();
        let mut argv = ping.to_vec();
        argv.push(&ip);
        let delivered = self
            .ssh(config, &from, &argv, Duration::from_secs(10))
            .await?;
        ensure!(
            delivered.status.success(),
            "guest fabric ping failed: {} {}",
            String::from_utf8_lossy(&delivered.stdout),
            String::from_utf8_lossy(&delivered.stderr)
        );
        let links: Vec<_> = self
            .fabric
            .simulation
            .links()
            .filter(|l| {
                l.interfaces.iter().any(|id| {
                    self.fabric
                        .simulation
                        .interface(id)
                        .is_ok_and(|i| i.node == to.node_id)
                })
            })
            .map(|l| l.id.clone())
            .collect();
        ensure!(!links.is_empty(), "partition target has no physical links");
        for link in &links {
            self.fabric.set_link_up(link, false).await?;
        }
        self.fabric.route_guests(&self.plan.guests)?;
        let blocked = self
            .ssh(config, &from, &argv, Duration::from_secs(10))
            .await?;
        for link in &links {
            self.fabric.set_link_up(link, true).await?;
        }
        self.fabric.route_guests(&self.plan.guests)?;
        ensure!(
            !blocked.status.success(),
            "guest traffic bypassed partitioned simulator fabric"
        );
        ensure!(
            self.ssh(config, &from, &argv, Duration::from_secs(10))
                .await?
                .status
                .success(),
            "guest fabric did not recover"
        );
        Ok(
            json!({"ok":true,"scope":"vm-plumbing-only","guests":self.plan.guests,"isolated_filesystems":true,"fabric_ping":true,"partition_blocked":true,"recovery":true,"gpu_tests_run":false}),
        )
    }
}
