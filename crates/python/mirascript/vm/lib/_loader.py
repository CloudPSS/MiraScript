# -*- coding: utf-8 -*-

from ..types.context import VmSharedContext
from . import vm_global
from ._helpers_utils import wrap_entry

lib_keys = []

# 注册 global 下所有导出到 VmSharedContext
for name in dir(vm_global):
    if name.startswith("_"):
        continue
    value = getattr(vm_global, name)
    VmSharedContext[name] = wrap_entry(name, value, "global")
    lib_keys.append(name)


lib = vm_global
print(
    "Loaded global library:",
    ", ".join(lib_keys),
)
