"""Dagger 0.21.8 pipeline; the same graph runs locally and in GitHub Actions."""
import json
import dagger
from dagger import dag, function, object_type


@object_type
class DatacenterCi:
    def runner(self, source: dagger.Directory, run_id: str) -> dagger.Container:
        clab=dag.container().from_('ghcr.io/srl-labs/clab@sha256:4b836d22c04fc0e9315f99648b7399e4859b71208c63eea00af488086d95fe12').file('/usr/bin/containerlab')
        return (dag.container().from_('ubuntu@sha256:224a1869083a311ef3f13648a154ba79832fbef6364d31493642ca03082da254')
                .with_exec(['bash','-ec','apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends ca-certificates docker.io iproute2 iputils-ping iptables openssh-client jq python3 python3-pip python3-venv sudo git && rm -rf /var/lib/apt/lists/*'])
                .with_file('/usr/local/bin/containerlab',clab)
                .with_directory('/workspace',source,exclude=['.git','**/.state','**/.venv','**/collections','**/__pycache__','**/.pytest_cache','**/.hypothesis','artifacts'])
                .with_workdir('/workspace')
                .with_exec(['python3','-m','venv','/opt/datacenter-ci'])
                .with_env_variable('PATH','/opt/datacenter-ci/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin')
                .with_exec(['pip','install','--no-cache-dir','-r','ci/requirements.txt'])
                .with_env_variable('ANSIBLE_COLLECTIONS_PATH','/opt/collections')
                .with_exec(['ansible-galaxy','collection','install','-r','cheatsheets/ex457/v6/requirements.yml','-p','/opt/collections'])
                .with_env_variable('ANSIBLE_NOCOLOR','1')
                .with_env_variable('PYTHONDONTWRITEBYTECODE','1')
                .with_env_variable('HYPOTHESIS_PROFILE','ci')
                .with_env_variable('CI_RUN_ID',run_id)
                .with_exec(['python','ci/run_ci.py'],insecure_root_capabilities=True))

    @function
    def artifacts(self, source: dagger.Directory, run_id: str) -> dagger.Directory:
        """Export reports on either outcome; this function alone is NOT a pass gate."""
        return self.runner(source,run_id).directory('/workspace/artifacts')

    @function
    async def ci(self, source: dagger.Directory, run_id: str) -> str:
        """Require every mandatory gate from this exact fresh run to pass."""
        result=json.loads(await self.runner(source,run_id).file('/workspace/artifacts/results.json').contents())
        if result['status']!='PASS':raise RuntimeError('Required CI gate failed: '+json.dumps(result))
        return json.dumps(result,indent=2)
