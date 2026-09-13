# 03 — VS Code, shell, SSH and Git

## Development exercise

Open this directory with `code .`. Create a scratch playbook in VS Code, save it, and run its syntax check in the integrated terminal. Use Source Control to inspect the diff, stage one file, commit, and compare the commit to its parent. Use the SSH command in chapter 01 to work with a router.

```bash
git status
git switch -c practice/ex457-change
ansible-playbook --syntax-check playbooks/show_version.yml
ansible-playbook playbooks/show_version.yml --limit spines
git diff
git add playbooks/show_version.yml
git diff --cached
git commit -m 'Practice FRR version inspection'
git fetch origin
git log --oneline --graph -8
```

For the standalone ZIP, initialize a local repository with `git init` before the branch exercise and configure your own name/email. It has no remote until you add one. Exercise a merge conflict on disposable branches: edit the same line twice, merge, resolve the markers, syntax-check and commit. Acceptance: explain the staged versus working-tree diff and identify the final commit.

[VS Code Source Control](https://code.visualstudio.com/docs/sourcecontrol/overview). [AAP VS Code setup](https://docs.redhat.com/en/documentation/red_hat_ansible_automation_platform/2.6/install-proc_devtools_install_vsc). [Git tutorial](https://git-scm.com/docs/gittutorial).

## Playbook pattern

Inspect the complete [show-version playbook](playbooks/show_version.yml): target group, `gather_facts: false`, model/trust pre-tasks, FQCN, `register`, display and `always` cleanup. Create a second command task, run with `--limit leaf1`, and explain the recap. `--syntax-check` parses structure; `--check` is module-dependent and cannot prove reachability or convergence.

[Ansible playbooks](https://docs.ansible.com/projects/ansible-core/2.19/playbook_guide/playbooks_intro.html).

