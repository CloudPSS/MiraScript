from __future__ import annotations
from typing_extensions import TypeAlias, Literal
from enum import Enum

kVmScript = "mirascript.vm.script"
kVmFunction = "mirascript.vm.function"
kVmContext = "mirascript.vm.context"


VM_ARRAY_MAX_LENGTH = 0x100_0000
# 16 M


class _Uninitialized(Enum):
    _ = object()

    def __bool__(self) -> bool:
        return self is not _Uninitialized._


Uninitialized: VmUninitialized = _Uninitialized._
"""Mirascript 虚拟机内的未初始化变量"""

VmUninitialized: TypeAlias = Literal[_Uninitialized._]
"""Mirascript 虚拟机内的未初始化变量"""

__all__ = [
    "kVmScript",
    "kVmFunction",
    "kVmContext",
    "VM_ARRAY_MAX_LENGTH",
    "Uninitialized",
    "VmUninitialized",
]
