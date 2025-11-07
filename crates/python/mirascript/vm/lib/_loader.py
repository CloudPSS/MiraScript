# -*- coding: utf-8 -*-
from ..lib._helpers import VmLibWrapper
from ..types import VmFunction, VmModule
from ..types.context import VmSharedContext
# from ._helpers import VmLib
import importlib
from . import vm_global
# 动态导入 global/index.py

def wrap_entry(name: str, value:VmLibWrapper):
    if callable(value):
        # Python 函数名不可直接更改，跳过重命名
        return VmFunction(value,{"isLib": True,
            "injectCp": True,})
    else:
        return value

# 注册 global 下所有导出到 VmSharedContext
for name in dir(vm_global):
    if name.startswith('_'):
        continue
    value = getattr(vm_global, name)
    VmSharedContext[name] = wrap_entry(name, value)

# def create_module(name: str, lib: dict) -> VmModule:
# 	mod = {}
# 	for key, value in lib.items():
# 		mod[key] = wrap_entry(key, value)
# 	return VmModule(name, mod)

lib = vm_global
