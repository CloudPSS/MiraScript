from __future__ import annotations
from typing_extensions import TypeAlias, Literal, Final
from enum import Enum

kVmScript: Final = "mirascript.vm.script"
kVmFunction: Final = "mirascript.vm.function"
kVmContext: Final = "mirascript.vm.context"


VM_ARRAY_MAX_LENGTH: Final = 0x100_0000
# 16 M


class _Uninitialized(Enum):
    _ = object()

    def __bool__(self) -> bool:
        return self is not _Uninitialized._

    def __repr__(self) -> str:
        return "<uninitialized>"

    def __str__(self) -> str:
        return "<uninitialized>"


Uninitialized: Final[VmUninitialized] = _Uninitialized._
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
