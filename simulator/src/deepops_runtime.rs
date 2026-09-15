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
use std::{
    fs::{self, File},
    process::Stdio,
    time::Duration,
};
use tokio::process::Command;

pub fn run(config: DeepOpsConfig, smoke: bool) -> Result<()> {
    let (sim, plan) = config.plan()?;
    crate::vm::preflight(&config, !smoke)?;
    if !smoke {
        verify_sources(&config)?;
    }
    patchbay::init_userns()?;
    tokio::runtime::Builder::new_current_thread().enable_all().build()?.block_on(async {
        let mut lab=tokio::time::timeout(Duration::from_secs(config.boot_timeout_secs+120),VmLab::boot(&config,&sim,plan)).await.context("VM boot deadline exceeded")??;
        let result=tokio::time::timeout(Duration::from_secs(config.timeout_secs),async {
            if smoke { lab.smoke(&config).await } else { deploy(&config,&mut lab).await }
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
fn verify_sources(config: &DeepOpsConfig) -> Result<()> {
    for (dir, expected) in [
        ("integrations/deepops/upstream", deepops::DEEPOPS_COMMIT),
        ("simulator/vendor/nccl", deepops::NCCL_COMMIT),
        ("simulator/vendor/nccl-tests", deepops::NCCL_TESTS_COMMIT),
    ] {
        let dir = config.repository.join(dir);
        ensure!(
            checked("git", &["-C", path(&dir)?, "rev-parse", "HEAD"])?.trim() == expected,
            "upstream revision mismatch: {}",
            dir.display()
        );
        ensure!(
            checked(
                "git",
                &[
                    "-C",
                    path(&dir)?,
                    "status",
                    "--porcelain",
                    "--untracked-files=no"
                ]
            )?
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
fn prepare_config(config: &DeepOpsConfig, plan: &DeepOpsPlan) -> Result<()> {
    let upstream = config.repository.join("integrations/deepops/upstream");
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
    // git archive uses the verified commit, never an uncommitted substitute library.
    for (source, revision, name) in [
        ("simulator/vendor/nccl", deepops::NCCL_COMMIT, "nccl"),
        (
            "simulator/vendor/nccl-tests",
            deepops::NCCL_TESTS_COMMIT,
            "nccl-tests",
        ),
    ] {
        checked(
            "git",
            &[
                "-C",
                path(&config.repository.join(source))?,
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
        .current_dir(config.repository.join("integrations/deepops/upstream"));
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
    cmd.stdin(Stdio::null())
        .stdout(File::create(&out)?)
        .stderr(File::create(directory.join(format!("{name}.stderr")))?)
        .kill_on_drop(true);
    let status = lab
        .fabric
        .device(&lab.plan.provisioner_id)?
        .spawn_command(cmd)?
        .wait()
        .await?;
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
async fn deploy(config: &DeepOpsConfig, lab: &mut VmLab) -> Result<Value> {
    prepare_config(config, &lab.plan)?;
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
                path(
                    &config
                        .repository
                        .join("integrations/deepops/prepare-nccl.yml"),
                )?,
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
        json!({"ok":true,"scope":"deepops-slurm-native-nccl","sources":{"deepops":deepops::DEEPOPS_COMMIT,"nccl":deepops::NCCL_COMMIT,"nccl_tests":deepops::NCCL_TESTS_COMMIT},"doctor":doctor,"slurm":slurm,"fabric":plumbing,"nccl_tests":reports,"gpu_tests_run":true}),
    )
}
