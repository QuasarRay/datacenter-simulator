"""Create a learner-only Rustlings community workspace; never copy trainer oracles."""
import json
from pathlib import Path
import shutil
import sys

ROOT=Path(__file__).resolve().parents[1]

def export(destination):
    destination=Path(destination).resolve()
    destination.mkdir(parents=True,exist_ok=False)
    for name in ('exercises','controllers','prompts'):(destination/name).mkdir()
    blueprint=ROOT/'education/ncp-metablueprint'
    curriculum=json.loads((blueprint/'curriculum.json').read_text())
    info=['format_version = 1','welcome_message = "NCP integrated incidents: use the Python controller and the supplied lab interfaces."',
          'final_message = "Local workflow complete. Only the trainer evidence ledger establishes NCP-Metablueprint mastery."']
    bins=[]
    for unit in curriculum:
        for exercise in unit['exercises']:
            name=exercise['id'].lower()
            info+=['','[[exercises]]',f'name = "{name}"','test = false',
                   f'input_files = ["controllers/{name}.py"]',
                   f'hint = "Read prompts/{exercise["id"]}.md and the preceding guided project. Diagnose from observations; no solution is embedded."']
            bins.append(f'  {{ name = "{name}", path = "exercises/{name}.rs" }},')
            (destination/'exercises'/f'{name}.rs').write_text(f'''// Run the learner controller; independent trainer grading follows this process.
fn main() -> Result<(), Box<dyn std::error::Error>> {{
    let python = std::env::var("NCP_PYTHON").unwrap_or_else(|_| "python3".into());
    let controller = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("controllers/{name}.py");
    let status = std::process::Command::new(python).arg(controller).status()?;
    if !status.success() {{ return Err("controller failed; no assessment credit".into()); }}
    Ok(())
}}
''')
            (destination/'controllers'/f'{name}.py').write_text(f'''"""Implement incident {exercise['id']} using the assigned guest/API credentials.

Read prompts/{exercise['id']}.md. Observe, predict, apply, measure and recover.
Do not create trainer evidence files or infer physical execution from KWOK status.
"""
raise NotImplementedError("Supply your controller for {exercise['id']}")
''')
            shutil.copyfile(blueprint/exercise['prompt'],destination/'prompts'/f'{exercise["id"]}.md')
    (destination/'info.toml').write_text('\n'.join(info)+'\n')
    (destination/'Cargo.toml').write_text('''bin = [
'''+ '\n'.join(bins)+'''
]
[package]
name = "ncp-learner-workshop"
version = "0.1.0"
edition = "2024"
publish = false
[workspace]
''')
    (destination/'README.md').write_text('''# NCP independent incidents

Use the trainer-provided Rustlings executable with this community workspace.
Edit `controllers/EXERCISE.py`; the Rust file is a small process adapter.
Prompts describe symptoms and required outcomes. They do not disclose faults
or expected configurations. The trainer supplies scoped lab credentials and
the course/project documents separately.

The trainer must enforce external grading outside the learner's OS identity.
Editing local progress, a controller or this adapter cannot establish mastery.
The exported workspace contains no grading configuration, evidence, product
credentials or Incus socket. A local-only run is practice, not certification.
''')
    return destination

if __name__=='__main__':
    if len(sys.argv)!=2:raise SystemExit('provide a new learner workspace directory')
    print(export(sys.argv[1]))
