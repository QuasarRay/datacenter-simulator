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

//! Supervise DeepOps's own executables and native nccl-tests in isolated guests.
//! Python is an external upstream deployment tool; there is no language FFI.
use crate::{
    deepops::{self, COLLECTIVES, DeepOpsConfig, DeepOpsPlan},
    vm::{VmLab, checked, path},
};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Stdio, time::Duration};
use tokio::process::Command;

pub fn run(config: DeepOpsConfig, smoke: bool) -> Result<()> {
    let (sim, plan) = config.plan()?;
    crate::vm::preflight(&config, !smoke)?;
    verify_sources(&config)?;
    // Host permissions are needed only to create the caller's requested state directory.
    // After entering the user namespace all writes are inside this owned directory.
    fs::create_dir(&config.state_dir).context("create run state directory")?;
    fs::set_permissions(&config.state_dir, fs::Permissions::from_mode(0o700))?;
    fs::create_dir(config.state_dir.join("reports"))?;
    fs::create_dir(config.state_dir.join("guests"))?;
    stage_deepops(&config)?;
    prepare_config(&config, &plan)?;
    patchbay::init_userns()?;
    tokio::runtime::Builder::new_current_thread().enable_all().build()?.block_on(async {
        let mut lab=tokio::time::timeout(Duration::from_secs(config.boot_timeout_secs+120),VmLab::boot(&config,&sim,plan)).await.context("VM boot deadline exceeded")??;
        let result=tokio::time::timeout(Duration::from_secs(config.timeout_secs),async {
            if smoke {
                let doctor = check_deepops(&config, &lab).await?;
                let mut report = lab.smoke(&config).await?;
                report["doctor"] = doctor;
                Ok(report)
            } else { deploy(&config,&mut lab).await }
        }).await.context("deployment/test deadline exceeded").and_then(|v|v);
        lab.shutdown().await;
        let report=match &result {
            Ok(report)=>report.clone(),
            Err(error)=>json!({"ok":false,"scope":if smoke {"vm-plumbing-only"} else {"deepops-slurm-native-nccl"},"error":format!("{error:#}")}),
        };
        fs::write(config.state_dir.join("reports/result.json"),serde_json::to_vec_pretty(&report)?)?;
        println!("{}",serde_json::to_string_pretty(&report)?);
        result.map(|_|())
    })
}
fn source_git(directory: &Path, args: &[&str]) -> Result<String> {
    let directory = fs::canonicalize(directory)?;
    // A sudo-launched runner still reads the explicitly selected, pinned checkout.
    // Trust is scoped to this command and directory; no global Git config is changed.
    let trusted = format!("safe.directory={}", path(&directory)?);
    let mut command = vec!["-c", &trusted, "-C", path(&directory)?];
    command.extend_from_slice(args);
    checked("git", &command)
}
fn verify_sources(config: &DeepOpsConfig) -> Result<()> {
    for (dir, expected) in [
        ("integrations/deepops/upstream", deepops::DEEPOPS_COMMIT),
        ("simulator/vendor/nccl", deepops::NCCL_COMMIT),
        ("simulator/vendor/nccl-tests", deepops::NCCL_TESTS_COMMIT),
    ] {
        let dir = fs::canonicalize(config.repository.join(dir))?;
        ensure!(
            source_git(&dir, &["rev-parse", "HEAD"])?.trim() == expected,
            "upstream revision mismatch: {}",
            dir.display()
        );
        ensure!(
            source_git(&dir, &["status", "--porcelain", "--untracked-files=no"])?
                .trim()
                .is_empty(),
            "upstream has tracked changes: {}",
            dir.display()
        );
    }
    ensure!(
        config.ansible_bin.join("ansible-playbook").is_file(),
        "run the pinned DeepOps scripts/setup.sh first"
    );
    Ok(())
}
fn integration(config: &DeepOpsConfig) -> std::path::PathBuf {
    config.state_dir.join("integration")
}
fn stage_deepops(config: &DeepOpsConfig) -> Result<()> {
    // The caller's checkout may be under a private home directory. Copy only
    // deployment sources/dependencies while host read permissions are available.
    // New copies belong to this run; no checkout permissions are modified.
    let original = config.repository.join("integrations/deepops");
    let staged = integration(config);
    fs::create_dir(&staged)?;
    fs::create_dir(staged.join("upstream"))?;
    let upstream = original.join("upstream");
    let archive = staged.join("deepops.tar");
    source_git(
        &upstream,
        &[
            "archive",
            "--format=tar",
            "-o",
            path(&archive)?,
            deepops::DEEPOPS_COMMIT,
        ],
    )?;
    checked(
        "tar",
        &[
            "-xf",
            path(&archive)?,
            "-C",
            path(&staged.join("upstream"))?,
        ],
    )?;
    fs::remove_file(archive)?;
    let gitlink = source_git(
        &upstream,
        &["ls-tree", deepops::DEEPOPS_COMMIT, "submodules/kubespray"],
    )?;
    let revision = gitlink
        .split_whitespace()
        .nth(2)
        .context("missing pinned Kubespray gitlink")?;
    let archive = staged.join("kubespray.tar");
    source_git(
        &upstream.join("submodules/kubespray"),
        &["archive", "--format=tar", "-o", path(&archive)?, revision],
    )?;
    fs::create_dir_all(staged.join("upstream/submodules/kubespray"))?;
    checked(
        "tar",
        &[
            "-xf",
            path(&archive)?,
            "-C",
            path(&staged.join("upstream/submodules/kubespray"))?,
        ],
    )?;
    fs::remove_file(archive)?;
    let galaxy = config.ansible_bin.join("ansible-galaxy");
    let requirements = staged.join("upstream/roles/requirements.yml");
    checked(
        path(&galaxy)?,
        &[
            "role",
            "install",
            "-r",
            path(&requirements)?,
            "-p",
            path(&staged.join("upstream/roles/galaxy"))?,
        ],
    )?;
    checked(
        path(&galaxy)?,
        &[
            "collection",
            "install",
            "-r",
            path(&requirements)?,
            "-p",
            path(&staged.join("upstream/collections"))?,
        ],
    )?;
    for entry in ["prepare-nccl.yml", "files"] {
        checked(
            "cp",
            &[
                "-R",
                "--no-preserve=ownership",
                path(&original.join(entry))?,
                path(&staged.join(entry))?,
            ],
        )?;
    }
    Ok(())
}
fn prepare_config(config: &DeepOpsConfig, plan: &DeepOpsPlan) -> Result<()> {
    let upstream = integration(config).join("upstream");
    let cfg = config.state_dir.join("config");
    fs::create_dir(&cfg)?;
    fs::create_dir(config.state_dir.join("private-logs"))?;
    for group in ["all", "slurm-cluster"] {
        let dir = cfg.join("group_vars").join(group);
        fs::create_dir_all(&dir)?;
        fs::copy(
            upstream.join(format!("config.example/group_vars/{group}.yml")),
            dir.join("00-upstream.yml"),
        )?;
    }
    fs::write(
        cfg.join("inventory.json"),
        serde_json::to_vec_pretty(&plan.inventory)?,
    )?;
    fs::write(
        cfg.join("group_vars/all/99-simulator.json"),
        serde_json::to_vec_pretty(
            &json!({"ansible_ssh_private_key_file":config.state_dir.join("id_ed25519")}),
        )?,
    )?;
    let options = json!({"hosts_network_interface":"simnccl", "deepops_disable_cloud_init":false,
        "slurm_allow_ssh_user":["deepops","root"], "slurm_cluster_install_openmpi":true,"openmpi_version":"4.1.8",
        "slurm_enable_nfs_server":false,"slurm_enable_nfs_client_nodes":false,"slurm_install_lmod":false,"slurm_install_hpcsdk":false,
        "slurm_install_enroot":false,"slurm_install_pyxis":false,"slurm_install_nhc":true,"slurm_enable_container_registry":false,
        "slurm_enable_monitoring":false,"slurm_enable_rsyslog_server":false,"slurm_enable_rsyslog_client":false,
        "install_dcgm":false,"chrony_install":false,"cuda_version":"cuda-toolkit-12-8","slurm_manage_gpus":true,"slurm_autodetect_nvml":true,
        "slurm_cluster_install_cuda":true,"slurm_cluster_install_nvidia_driver":true,
        "slurm_password":uuid::Uuid::new_v4().to_string(),"slurm_db_password":uuid::Uuid::new_v4().to_string()});
    fs::write(
        cfg.join("group_vars/slurm-cluster/99-simulator.json"),
        serde_json::to_vec_pretty(&options)?,
    )?;
    fs::write(
        cfg.join("ansible.cfg"),
        format!(
            "[defaults]\ninventory = {}\nroles_path = {}/roles/galaxy:{}/roles:{}/submodules/kubespray/roles\ncollections_paths = {}/collections\nlibrary = {}/submodules/kubespray/library\nprivate_key_file = {}\nhost_key_checking = True\nfact_caching = memory\nretry_files_enabled = False\nforks = 8\n[ssh_connection]\nssh_args = -o ControlMaster=no -o IdentitiesOnly=yes -o StrictHostKeyChecking=accept-new -o UserKnownHostsFile={}\n",
            path(&cfg.join("inventory.json"))?,
            path(&upstream)?,
            path(&upstream)?,
            path(&upstream)?,
            path(&upstream)?,
            path(&upstream)?,
            path(&config.state_dir.join("id_ed25519"))?,
            path(&config.state_dir.join("known_hosts"))?
        ),
    )?;
    // The upstream doctor identifies its repository root by this file. Use the
    // same isolated inventory here as in ANSIBLE_CONFIG, including for subprocesses.
    fs::copy(cfg.join("ansible.cfg"), upstream.join("ansible.cfg"))?;
    // git archive uses the verified commit, never an uncommitted substitute library.
    for (source, revision, name) in [
        ("simulator/vendor/nccl", deepops::NCCL_COMMIT, "nccl"),
        (
            "simulator/vendor/nccl-tests",
            deepops::NCCL_TESTS_COMMIT,
            "nccl-tests",
        ),
    ] {
        source_git(
            &config.repository.join(source),
            &[
                "archive",
                "--format=tar.gz",
                "-o",
                path(&config.state_dir.join(format!("{name}.tar.gz")))?,
                revision,
            ],
        )?;
    }
    let vars = json!({"simulator_nccl_archive":config.state_dir.join("nccl.tar.gz"),"simulator_nccl_tests_archive":config.state_dir.join("nccl-tests.tar.gz"),"simulator_source_revisions":{"deepops":deepops::DEEPOPS_COMMIT,"nccl":deepops::NCCL_COMMIT,"nccl_tests":deepops::NCCL_TESTS_COMMIT}});
    fs::write(cfg.join("sources.json"), serde_json::to_vec_pretty(&vars)?)?;
    Ok(())
}
fn ansible_command(config: &DeepOpsConfig, tool: &str, args: &[&str]) -> Result<Command> {
    let mut cmd = Command::new(config.ansible_bin.join(tool));
    cmd.args(args)
        .current_dir(integration(config).join("upstream"));
    cmd.env_clear()
        .env(
            "PATH",
            format!(
                "{}:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
                path(&config.ansible_bin)?
            ),
        )
        .env("LANG", "C.UTF-8")
        .env(
            "ANSIBLE_CONFIG",
            config.state_dir.join("config/ansible.cfg"),
        )
        .env("DEEPOPS_CONFIG_DIR", config.state_dir.join("config"))
        .env("ANSIBLE_LOCAL_TEMP", config.state_dir.join("ansible-tmp"));
    Ok(cmd)
}
async fn stage(
    config: &DeepOpsConfig,
    lab: &VmLab,
    name: &str,
    mut cmd: Command,
    private: bool,
) -> Result<std::path::PathBuf> {
    eprintln!("DeepOps stage: {name}");
    let directory = config
        .state_dir
        .join(if private { "private-logs" } else { "reports" });
    let out = directory.join(format!("{name}.stdout"));
    let (stdout, out_done) = crate::bounded_log::capture(&out)?;
    let (stderr, err_done) =
        crate::bounded_log::capture(&directory.join(format!("{name}.stderr")))?;
    cmd.stdin(Stdio::null())
        .stdout(stdout)
        .stderr(stderr)
        .kill_on_drop(true);
    cmd.process_group(0);
    let program = cmd.as_std().get_program().to_string_lossy().to_string();
    let mut child = lab
        .fabric
        .device(&lab.plan.provisioner_id)?
        .spawn_command(cmd)
        .with_context(|| format!("start {program} for {name}"))?;
    let _group = crate::vm::ProcessGroup(child.id().context("stage process has no PID")?);
    let status = child.wait().await?;
    out_done.finish()?;
    err_done.finish()?;
    ensure!(
        status.success(),
        "{name} failed ({status}); inspect {}",
        directory.display()
    );
    Ok(out)
}
async fn json_stage(
    config: &DeepOpsConfig,
    lab: &VmLab,
    name: &str,
    cmd: Command,
    private: bool,
) -> Result<Value> {
    let file = stage(config, lab, name, cmd, private).await?;
    serde_json::from_slice(&fs::read(file)?).with_context(|| format!("{name} did not return JSON"))
}
async fn check_deepops(config: &DeepOpsConfig, lab: &VmLab) -> Result<Value> {
    // Run upstream tests without reimplementing the upstream tools or their protocols.
    for (name, dir) in [
        ("deepops-validation-unit", "scripts/validation/tests"),
        ("deepops-firmware-unit", "roles/nvidia-dgx-firmware/tests"),
    ] {
        stage(
            config,
            lab,
            name,
            ansible_command(
                config,
                "python3",
                &["-m", "unittest", "discover", "-s", dir, "-v"],
            )?,
            false,
        )
        .await?;
    }
    let inventory = json_stage(
        config,
        lab,
        "resolved-inventory",
        ansible_command(config, "ansible-inventory", &["--list"])?,
        true,
    )
    .await?;
    deepops::validate_inventory(&inventory, &lab.plan.guests)?;
    let doctor = json_stage(
        config,
        lab,
        "deepops-doctor",
        ansible_command(
            config,
            "python3",
            &[
                "scripts/validation/deepops_doctor.py",
                "--inventory",
                path(&config.state_dir.join("config/inventory.json"))?,
                "--remote",
                "--json",
            ],
        )?,
        false,
    )
    .await?;
    ensure!(
        doctor["ok"] == true,
        "DeepOps doctor rejected deployment: {doctor}"
    );
    Ok(doctor)
}
async fn deploy(config: &DeepOpsConfig, lab: &mut VmLab) -> Result<Value> {
    let doctor = check_deepops(config, lab).await?;
    stage(
        config,
        lab,
        "deepops-deploy",
        ansible_command(
            config,
            "ansible-playbook",
            &[
                "--flush-cache",
                "-l",
                "slurm-cluster",
                "playbooks/slurm-cluster.yml",
            ],
        )?,
        true,
    )
    .await?;
    stage(
        config,
        lab,
        "prepare-nccl",
        ansible_command(
            config,
            "ansible-playbook",
            &[
                "-l",
                "slurm-cluster",
                path(&integration(config).join("prepare-nccl.yml"))?,
                "-e",
                &format!("@{}", path(&config.state_dir.join("config/sources.json"))?),
            ],
        )?,
        true,
    )
    .await?;
    let controller = &lab.plan.guests[0];
    let slurm = json_stage(
        config,
        lab,
        "deepops-slurm",
        lab.ssh_command(
            config,
            controller,
            &[
                "env",
                "-i",
                "PATH=/usr/local/bin:/usr/bin:/bin",
                "python3",
                "/usr/local/bin/deepops-validate-slurm",
                "--json",
            ],
        )?,
        false,
    )
    .await?;
    deepops::validate_slurm(&slurm, &config.compute)?;
    // Prove that guest traffic is still subject to physical modeled links after deployment.
    let plumbing = lab.smoke(config).await?;
    let partition = partition_nccl(config, lab).await?;
    let mut reports = serde_json::Map::new();
    let nodes = config.compute.join(",");
    let count = config.compute.len().to_string();
    for collective in COLLECTIVES {
        let controller = &lab.plan.guests[0];
        let cmd = lab.ssh_command(
            config,
            controller,
            &[
                "env",
                "-i",
                "PATH=/usr/local/bin:/usr/bin:/bin",
                "/usr/local/bin/srun",
                "--mpi=pmix",
                "--nodes",
                &count,
                "--ntasks",
                &count,
                "--ntasks-per-node=1",
                "--gpus-per-task=1",
                "--cpu-bind=none",
                "--nodelist",
                &nodes,
                "--time=00:05:00",
                "--immediate=30",
                "--kill-on-bad-exit=1",
                "--export=NIL",
                "/opt/simulator/nccl-rank",
                collective,
            ],
        )?;
        stage(config, lab, &format!("nccl-{collective}"), cmd, false).await?;
        let mut found = None;
        for guest in &lab.plan.guests[1..] {
            let output = lab
                .ssh(
                    config,
                    guest,
                    &["cat", &format!("/var/tmp/simulator-nccl/{collective}.json")],
                    Duration::from_secs(15),
                )
                .await?;
            if output.status.success() {
                ensure!(
                    found.is_none(),
                    "multiple rank-zero reports for {collective}"
                );
                let report: Value = serde_json::from_slice(&output.stdout)?;
                deepops::validate_nccl(&report, collective, &config.compute)?;
                fs::write(
                    config
                        .state_dir
                        .join(format!("reports/nccl-{collective}.json")),
                    serde_json::to_vec_pretty(&report)?,
                )?;
                found = Some(report);
            }
        }
        let report = found.context("NCCL rank zero produced no report")?;
        reports.insert(collective.to_string(),json!({"ok":true,"rows":report["results"].as_array().map(Vec::len),"ranks":config.compute.len()}));
    }
    Ok(
        json!({"ok":true,"scope":"deepops-slurm-native-nccl","sources":{"deepops":deepops::DEEPOPS_COMMIT,"nccl":deepops::NCCL_COMMIT,"nccl_tests":deepops::NCCL_TESTS_COMMIT},"doctor":doctor,"slurm":slurm,"fabric":plumbing,"nccl_tests":reports,"in_flight_partition":partition,"gpu_tests_run":true}),
    )
}

/// Wait for a real nccl-tests correctness row, cut the modeled data fabric while
/// repeated NCCL cycles are running, require srun failure, then restore before recovery tests.
async fn partition_nccl(config: &DeepOpsConfig, lab: &mut VmLab) -> Result<Value> {
    let controller = lab.plan.guests[0].clone();
    let target = lab.plan.guests[2].node_id.clone();
    let links: Vec<_> = lab
        .fabric
        .simulation
        .links()
        .filter(|l| {
            l.spec.up
                && l.interfaces.iter().any(|id| {
                    lab.fabric
                        .simulation
                        .interface(id)
                        .is_ok_and(|i| i.node == target)
                })
        })
        .map(|l| l.id.clone())
        .collect();
    ensure!(
        !links.is_empty(),
        "NCCL partition target has no active links"
    );
    let name = format!("simulator-partition-{}", uuid::Uuid::new_v4());
    let count = config.compute.len().to_string();
    let nodes = config.compute.join(",");
    let stdout = config.state_dir.join("reports/nccl-partition.stdout");
    let stderr = config.state_dir.join("reports/nccl-partition.stderr");
    let (out_pipe, out_done) = crate::bounded_log::capture(&stdout)?;
    let (err_pipe, err_done) = crate::bounded_log::capture(&stderr)?;
    let mut cmd = lab.ssh_command(
        config,
        &controller,
        &[
            "env",
            "-i",
            "PATH=/usr/local/bin:/usr/bin:/bin",
            "/usr/local/bin/srun",
            "--mpi=pmix",
            "--nodes",
            &count,
            "--ntasks",
            &count,
            "--ntasks-per-node=1",
            "--gpus-per-task=1",
            "--cpu-bind=none",
            "--nodelist",
            &nodes,
            "--time=00:03:00",
            "--immediate=30",
            "--kill-on-bad-exit=1",
            "--export=NIL",
            "--job-name",
            &name,
            "/opt/simulator/nccl-rank",
            "all_reduce",
            "partition",
        ],
    )?;
    cmd.stdout(out_pipe)
        .stderr(err_pipe)
        .stdin(Stdio::null())
        .process_group(0)
        .kill_on_drop(true);
    let mut child = lab
        .fabric
        .device(&lab.plan.provisioner_id)?
        .spawn_command(cmd)?;
    let _group = crate::vm::ProcessGroup(child.id().context("partition process has no PID")?);
    let mut guard = crate::vm::VmPartition::new(lab);
    let outcome = async {
        tokio::time::timeout(Duration::from_secs(90), async {
            loop {
                ensure!(
                    child.try_wait()?.is_none(),
                    "NCCL exited before fault injection"
                );
                let output = fs::read_to_string(&stdout)?;
                if output.lines().any(|line| {
                    let fields: Vec<_> = line.split_whitespace().collect();
                    fields.len() >= 13
                        && fields[0].parse::<u64>() == Ok(16 * 1024 * 1024)
                        && fields[8] == "0"
                        && fields[12] == "0"
                }) {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Ok::<_, anyhow::Error>(())
        })
        .await
        .context("no completed native NCCL row before partition deadline")??;
        guard.cut(&links)?;
        let status = tokio::time::timeout(Duration::from_secs(75), child.wait())
            .await
            .context("partitioned srun did not terminate")??;
        ensure!(
            !status.success(),
            "NCCL unexpectedly completed through the partition"
        );
        Ok::<_, anyhow::Error>(())
    }
    .await;
    let _ = child.start_kill();
    let _ = tokio::time::timeout(Duration::from_secs(2), child.wait()).await;
    let restore = guard.restore();
    // Cancel the uniquely named remote job even if SSH failed while carrying output.
    let cancel = lab
        .ssh(
            config,
            &controller,
            &["/usr/local/bin/scancel", "--name", &name],
            Duration::from_secs(15),
        )
        .await;
    restore?;
    let cancel = cancel?;
    ensure!(
        cancel.status.success(),
        "failed to cancel partition test job"
    );
    out_done.finish()?;
    err_done.finish()?;
    outcome?;
    Ok(
        json!({"scope":"deepops-slurm-native-nccl-socket","completed_row_before_cut":true,
        "partition_failed_running_job":true,"links_restored":true,"recovery_checked_by_following_nccl_tests":true}),
    )
}
