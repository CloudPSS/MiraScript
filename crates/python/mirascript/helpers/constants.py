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
    _ = type("VmUninitialized", (), {})()


VmUninitialized: TypeAlias = Literal[_Uninitialized._]
"""Mirascript 虚拟机内的未初始化变量"""

Uninitialized: VmUninitialized = _Uninitialized._
"""Mirascript 虚拟机内的未初始化变量"""
