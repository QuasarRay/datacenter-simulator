"""Execute the privileged integration-test lifecycle inside Dagger."""

from __future__ import annotations

import os
import signal
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOCKER_LOG = Path("/tmp/dockerd.log")


def run(command: list[str], *, check: bool = True) -> subprocess.CompletedProcess[str]:
    print(f"+ {' '.join(command)}", flush=True)
    return subprocess.run(
        command,
        cwd=ROOT,
        check=check,
        text=True,
        env={**os.environ, "ANSIBLE_NOCOLOR": "1"},
    )


def wait_for_docker(timeout: int = 90) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        result = subprocess.run(
            ["docker", "info"],
            cwd=ROOT,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        if result.returncode == 0:
            return
        time.sleep(1)

    if DOCKER_LOG.exists():
        print(DOCKER_LOG.read_text(errors="replace"), file=sys.stderr)
    raise RuntimeError("Docker daemon did not become ready")


def main() -> int:
    DOCKER_LOG.parent.mkdir(parents=True, exist_ok=True)

    with DOCKER_LOG.open("w") as log:
        dockerd = subprocess.Popen(
            [
                "dockerd",
                "--host=unix:///var/run/docker.sock",
                "--storage-driver=vfs",
            ],
            stdout=log,
            stderr=subprocess.STDOUT,
            text=True,
        )

    try:
        wait_for_docker()
        run(["docker", "version"])
        run(["containerlab", "version"])

        run(
            [
                "ansible-playbook",
                "-i",
                "inventory/localhost.ini",
                "playbooks/deploy.yml",
            ]
        )

        run(["python", "tests/datacenter_state.py"])
        return 0

    except Exception:
        print("\n--- diagnostic: docker ps -a ---", file=sys.stderr)
        run(["docker", "ps", "-a"], check=False)

        topology = ROOT / "build" / "gpu-dc.clab.yml"
        if topology.exists():
            print("\n--- diagnostic: containerlab inspect ---", file=sys.stderr)
            run(
                [
                    "containerlab",
                    "inspect",
                    "--topo",
                    str(topology),
                    "--format",
                    "json",
                ],
                check=False,
            )
        raise

    finally:
        run(
            [
                "ansible-playbook",
                "-i",
                "inventory/localhost.ini",
                "playbooks/destroy.yml",
            ],
            check=False,
        )

        if dockerd.poll() is None:
            dockerd.send_signal(signal.SIGTERM)
            try:
                dockerd.wait(timeout=15)
            except subprocess.TimeoutExpired:
                dockerd.kill()


if __name__ == "__main__":
    raise SystemExit(main())
