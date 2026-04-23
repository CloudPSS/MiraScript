"""
MiraScript Python Module

This module provides the main entry point for compiling MiraScript code
"""

from .mirascript import Config
from .diagnostics import Diagnostic
from .main import compile
from .version import __version__

__all__ = ["Config", "Diagnostic", "compile", "__version__"]
