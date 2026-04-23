"""
MiraScript Python Module

This module provides the main entry point for compiling MiraScript code
"""

from .mirascript import Config
from .diagnostics import Diagnostic
from .main import compile

__all__ = ["Config", "Diagnostic", "compile"]
