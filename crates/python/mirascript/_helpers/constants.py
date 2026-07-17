from __future__ import annotations
from typing_extensions import TypeAlias, Literal
from enum import Enum

kVmScript = "mirascript.vm.script"
kVmFunction = "mirascript.vm.function"
kVmFunctionProxy = "mirascript.vm.function.proxy"
kVmContext = "mirascript.vm.context"
kVmExtern = "mirascript.vm.extern"
kVmModule = "mirascript.vm.module"
kVmWrapper = "mirascript.vm.wrapper"


VM_ARRAY_MAX_LENGTH = 0x100_0000
# 16 M


class _Uninitialized(Enum):
    U = object()

    def __bool__(self) -> bool:
        return self is not _Uninitialized.U


Uninitialized: VmUninitialized = _Uninitialized.U
"""Mirascript 虚拟机内的未初始化变量"""

VmUninitialized: TypeAlias = Literal[_Uninitialized.U]
"""Mirascript 虚拟机内的未初始化变量"""
