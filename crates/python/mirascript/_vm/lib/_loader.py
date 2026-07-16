from inspect import ismodule
from typing import cast

from ..types.context import VmSharedContext
from ..types.types import Uninitialized, VmValue
from ..._helpers import types as checker
from .. import types
from . import vm_global
from ._helpers_utils import wrap_entry

# 注册 global 下所有导出到 VmSharedContext
for name in dir(vm_global):
    if name.startswith("_") or name.endswith("_"):
        continue
    value = getattr(vm_global, name)
    if ismodule(value):
        continue
    if callable(value) and name.lower() != name:
        continue
    if name == "Uninitialized" and value is Uninitialized:
        continue
    if hasattr(types, name) and getattr(types, name) == value:
        continue
    if hasattr(checker, name) and getattr(checker, name) == value:
        continue
    VmSharedContext[name] = wrap_entry(name, cast(VmValue, value), "global")


lib = vm_global
