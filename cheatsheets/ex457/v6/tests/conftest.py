"""Deterministic CI generation; local runs may select HYPOTHESIS_PROFILE=dev."""
import os
from pathlib import Path
import sys
from hypothesis import settings
ROOT = Path(__file__).resolve().parents[1]
sys.path[:0] = [str(ROOT/'filter_plugins'), str(ROOT/'tools'), str(ROOT/'tests')]
settings.register_profile('ci', max_examples=150, derandomize=True, deadline=None, print_blob=True)
settings.register_profile('dev', max_examples=30, deadline=None)
settings.load_profile(os.environ.get('HYPOTHESIS_PROFILE', 'ci'))
