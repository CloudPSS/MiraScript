"""
MiraScript Python Module

This module provides the main entry point for compiling MiraScript code
"""

from .mirascript import Config
from .diagnostics import Diagnostic
from .main import compile
__version__ = '0.0.26'
__all__ = ["Config", "Diagnostic", "compile","__version__"]
