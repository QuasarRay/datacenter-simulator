"""pyATS tests that compare the running Containerlab lab to declared intent."""

from __future__ import annotations

import ipaddress
import json
import re
import subprocess
from pathlib import Path
from typing import Any

import yaml
from pyats import aetest

ROOT = Path(__file__).resolve().parents[1]
INTENT_FILE = ROOT / "vars" / "datacenter.yml"


def command(args: list[str]) -> str:
    result = subprocess.run(
        args,
        cwd=ROOT,
        check=True,
        text=True,
        capture_output=True,
    )
    return result.stdout


def docker_inspect(container: str) -> dict[str, Any]:
    return json.loads(command(["docker", "inspect", container]))[0]


def container_name(lab_name: str, node_name: str) -> str:
    return f"clab-{lab_name}-{node_name}"


def parse_memory(value: str) -> int:
    match = re.fullmatch(r"(?i)(\d+(?:\.\d+)?)\s*([kmgt]?i?b?)", value.strip())
    if not match:
        raise ValueError(f"Unsupported memory value: {value}")

    number = float(match.group(1))
    unit = match.group(2).lower()
    factors = {
        "": 1,
        "b": 1,
        "k": 1024,
        "kb": 1024,
        "ki": 1024,
        "kib": 1024,
        "m": 1024**2,
        "mb": 1024**2,
        "mi": 1024**2,
        "mib": 1024**2,
        "g": 1024**3,
        "gb": 1024**3,
        "gi": 1024**3,
        "gib": 1024**3,
        "t": 1024**4,
        "tb": 1024**4,
        "ti": 1024**4,
        "tib": 1024**4,
    }
    return int(number * factors[unit])


def interface_addresses(container: str, interface: str) -> set[str]:
    data = json.loads(
        command(["docker", "exec", container, "ip", "-j", "address", "show", "dev", interface])
    )
    if not data:
        return set()

    addresses: set[str] = set()
    for addr in data[0].get("addr_info", []):
        if addr.get("family") != "inet":
            continue
        addresses.add(f"{addr['local']}/{addr['prefixlen']}")
    return addresses


class CommonSetup(aetest.CommonSetup):
    @aetest.subsection
    def load_intent(self, testscript):
        with INTENT_FILE.open() as handle:
            intent = yaml.safe_load(handle)["dc"]

        testscript.parameters["intent"] = intent
        testscript.parameters["lab_name"] = intent["name"]

    @aetest.subsection
    def require_running_lab(self, intent, lab_name):
        expected = {
            container_name(lab_name, node_name)
            for node_name in intent["nodes"]
        }
        actual = set(command(["docker", "ps", "--format", "{{.Names}}"]).splitlines())

        missing = sorted(expected - actual)
        if missing:
            self.failed(f"Expected Containerlab nodes are not running: {missing}")


class NodeStateMatchesIntent(aetest.Testcase):
    @aetest.test
    def node_properties(self, intent, lab_name):
        failures: list[str] = []

        for node_name, node in intent["nodes"].items():
            profile = intent["profiles"][node["role"]]
            name = container_name(lab_name, node_name)
            state = docker_inspect(name)

            if not state["State"]["Running"]:
                failures.append(f"{node_name}: container is not running")

            if state["Config"]["Image"] != profile["image"]:
                failures.append(
                    f"{node_name}: image={state['Config']['Image']} "
                    f"expected={profile['image']}"
                )

            labels = state["Config"].get("Labels") or {}
            expected_labels = {
                "role": str(node["role"]),
                "asn": str(node["asn"]),
                "loopback": str(node["loopback"]),
            }
            for key, expected_value in expected_labels.items():
                if labels.get(key) != expected_value:
                    failures.append(
                        f"{node_name}: label {key}={labels.get(key)!r} "
                        f"expected={expected_value!r}"
                    )

            network = intent["mgmt"]["network"]
            network_state = state["NetworkSettings"]["Networks"].get(network)
            if not network_state:
                failures.append(f"{node_name}: missing management network {network}")
            elif network_state.get("IPAddress") != node["mgmt_ip"]:
                failures.append(
                    f"{node_name}: mgmt IP={network_state.get('IPAddress')} "
                    f"expected={node['mgmt_ip']}"
                )

            expected_cpu = int(float(profile["cpu"]) * 1_000_000_000)
            actual_cpu = int(state["HostConfig"].get("NanoCpus") or 0)
            if actual_cpu != expected_cpu:
                failures.append(
                    f"{node_name}: NanoCPUs={actual_cpu} expected={expected_cpu}"
                )

            expected_memory = parse_memory(str(profile["memory"]))
            actual_memory = int(state["HostConfig"].get("Memory") or 0)
            if actual_memory != expected_memory:
                failures.append(
                    f"{node_name}: memory={actual_memory} expected={expected_memory}"
                )

            loopback = str(ipaddress.ip_interface(node["loopback"]))
            if loopback not in interface_addresses(name, "lo"):
                failures.append(
                    f"{node_name}: loopback {loopback} is not configured on lo"
                )

        if failures:
            self.failed("\n".join(failures))


class FabricLinksMatchIntent(aetest.Testcase):
    @aetest.test
    def link_interfaces_and_addresses(self, intent, lab_name):
        failures: list[str] = []

        for link in intent["links"]:
            if len(link["endpoints"]) != 2:
                failures.append(
                    f"{link['name']}: expected exactly two endpoints, "
                    f"found {len(link['endpoints'])}"
                )
                continue

            for endpoint in link["endpoints"]:
                node_name = endpoint["node"]
                name = container_name(lab_name, node_name)
                interface = endpoint["interface"]
                expected_address = str(ipaddress.ip_interface(endpoint["address"]))

                try:
                    addresses = interface_addresses(name, interface)
                except subprocess.CalledProcessError as exc:
                    failures.append(
                        f"{link['name']}: {node_name}:{interface} is missing "
                        f"or unreadable ({exc})"
                    )
                    continue

                if expected_address not in addresses:
                    failures.append(
                        f"{link['name']}: {node_name}:{interface} has {sorted(addresses)}, "
                        f"expected {expected_address}"
                    )

        if failures:
            self.failed("\n".join(failures))


class ContainerlabOperationalState(aetest.Testcase):
    @aetest.test
    def interfaces_are_up(self, intent):
        topology = ROOT / "build" / f"{intent['name']}.clab.yml"
        output = command(
            [
                "containerlab",
                "inspect",
                "interfaces",
                "--topo",
                str(topology),
                "--format",
                "json",
            ]
        )
        interface_data = json.loads(output)

        failures: list[str] = []
        expected_interfaces = {
            (container_name(intent["name"], endpoint["node"]), endpoint["interface"])
            for link in intent["links"]
            for endpoint in link["endpoints"]
        }

        actual: dict[tuple[str, str], str] = {}
        for node in interface_data:
            node_name = node.get("name")
            for interface in node.get("interfaces", []):
                actual[(node_name, interface.get("name"))] = interface.get("state")

        for key in sorted(expected_interfaces):
            if key not in actual:
                failures.append(f"{key[0]}:{key[1]} is absent from operational state")
            elif actual[key] != "up":
                failures.append(
                    f"{key[0]}:{key[1]} state={actual[key]!r}, expected='up'"
                )

        if failures:
            self.failed("\n".join(failures))


if __name__ == "__main__":
    aetest.main()
