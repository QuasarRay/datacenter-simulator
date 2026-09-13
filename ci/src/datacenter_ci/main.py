"""Portable CI pipeline for the datacenter simulator."""

import dagger
from dagger import dag, function, object_type


@object_type
class DatacenterCi:
    """Build an isolated Containerlab testbed and validate it with pyATS."""

    @function
    async def ci(self, source: dagger.Directory) -> str:
        """Render, deploy, test, and destroy the virtual datacenter."""

        containerlab = (
            dag.container()
            .from_("ghcr.io/srl-labs/clab:0.79.0")
            .file("/usr/bin/containerlab")
        )

        runner = (
            dag.container()
            .from_("ubuntu:24.04")
            .with_exec(
                [
                    "bash",
                    "-lc",
                    (
                        "apt-get update && "
                        "DEBIAN_FRONTEND=noninteractive apt-get install -y "
                        "--no-install-recommends "
                        "ca-certificates docker.io iproute2 iputils-ping jq "
                        "python3 python3-pip python3-venv sudo && "
                        "rm -rf /var/lib/apt/lists/*"
                    ),
                ]
            )
            .with_file("/usr/local/bin/containerlab", containerlab)
            .with_directory("/workspace", source)
            .with_workdir("/workspace")
            .with_exec(["python3", "-m", "venv", "/opt/datacenter-ci"])
            .with_exec(
                [
                    "/opt/datacenter-ci/bin/pip",
                    "install",
                    "--no-cache-dir",
                    "-r",
                    "ci/requirements.txt",
                ]
            )
            .with_env_variable(
                "PATH",
                (
                    "/opt/datacenter-ci/bin:/usr/local/sbin:/usr/local/bin:"
                    "/usr/sbin:/usr/bin:/sbin:/bin"
                ),
            )
            .with_env_variable("ANSIBLE_NOCOLOR", "1")
            .with_exec(
                [
                    "ansible-playbook",
                    "--syntax-check",
                    "-i",
                    "inventory/localhost.ini",
                    "playbooks/deploy.yml",
                ]
            )
            .with_exec(["python", "-m", "compileall", "-q", "ci", "tests"])
            .with_exec(
                ["python", "ci/run_ci.py"],
                insecure_root_capabilities=True,
            )
        )

        return await runner.stdout()
