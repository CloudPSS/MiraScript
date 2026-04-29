from typing_extensions import TypeAlias, Never

from .module import VmModule
from .function import VmFunction
from ...helpers.constants import Uninitialized, VmUninitialized

VmExtern: TypeAlias = Never
"""Mirascript 虚拟机内的外部对象，Python 环境暂不支持外部对象，因此该类型永远没有值"""

VmPrimitive: TypeAlias = "str | int | float | bool | None"
"""Mirascript 原始值"""

VmRecord: TypeAlias = "dict[str, VmValue]"
"""Mirascript 记录"""
VmArray: TypeAlias = "list[VmValue]"
"""Mirascript 数组"""

VmConst: TypeAlias = "VmPrimitive | VmRecord | VmArray"
"""Mirascript 虚拟机内的值语义值"""

VmImmutable: TypeAlias = "VmConst | VmModule | VmFunction"
"""Mirascript 虚拟机内的不可变值"""
VmValue: TypeAlias = "VmImmutable | VmExtern"
"""Mirascript 虚拟机内的合法值"""
VmAny: TypeAlias = "VmValue | VmUninitialized"
"""Mirascript 虚拟机内的值（包括未初始化变量）"""

__all__ = [
    "VmExtern",
    "Uninitialized",
    "VmUninitialized",
    "VmPrimitive",
    "VmRecord",
    "VmArray",
    "VmConst",
    "VmImmutable",
    "VmValue",
    "VmAny",
    "VmFunction",
]
