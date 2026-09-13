"""Load the shared verifier when Controller starts in a nested project path."""
import importlib.util
from pathlib import Path
spec = importlib.util.spec_from_file_location('ex457_fabric_impl', Path(__file__).resolve().parents[2] / 'filter_plugins/fabric.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
FilterModule = module.FilterModule
