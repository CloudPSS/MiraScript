"""
MiraScript Python Module

This module provides the main entry point for compiling MiraScript code
"""

from .compiler import compile, Diagnostic, VmScript, VmScriptLike
from .helpers.types import (
    is_vm_primitive,
    is_vm_const,
    is_vm_array,
    is_vm_record,
    is_vm_extern,
    is_vm_module,
    is_vm_any,
    is_vm_callable,
    is_vm_context,
    is_vm_function,
    is_vm_immutable,
    is_vm_script,
    is_vm_value,
    is_vm_wrapper,
)
from .helpers.constants import Uninitialized, VmUninitialized
from .helpers.convert import to_boolean, to_format, to_number, to_string
from .helpers.serialize import serialize, display
from .vm import (
    VmError,
    vm_function,
    VmModule,
    VmContext,
    VmAny,
    VmArray,
    VmConst,
    VmExtern,
    VmFunction,
    VmImmutable,
    VmPrimitive,
    VmRecord,
    VmValue,
    lib,
    operations,
    config_checkpoint,
)

__all__ = [
    "compile",
    "VmScript",
    "VmScriptLike",
    "VmContext",
    "VmModule",
    "VmError",
    "vm_function",
    "VmValue",
    "VmPrimitive",
    "VmRecord",
    "VmArray",
    "VmConst",
    "VmImmutable",
    "VmExtern",
    "Uninitialized",
    "VmUninitialized",
    "VmFunction",
    "VmAny",
]
