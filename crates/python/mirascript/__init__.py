"""
MiraScript Python Module

This module provides the main entry point for compiling MiraScript code
"""

from .mirascript import Config
from .emit import Context
from .diagnostics import Diagnostic
from .main import compile

__all__ = ["Config", "Context", "Diagnostic", "compile"]
