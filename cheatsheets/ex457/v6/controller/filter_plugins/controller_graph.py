from pathlib import Path
import importlib.util
spec = importlib.util.spec_from_file_location('ex457_controller_graph', Path(__file__).resolve().parents[2]/'filter_plugins/controller_graph.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
FilterModule = module.FilterModule
