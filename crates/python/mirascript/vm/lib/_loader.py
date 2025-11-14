# -*- coding: utf-8 -*-
from ..lib._helpers import VmLibWrapper

from ..types.context import VmSharedContext
# from ._helpers import VmLib
import importlib
from . import vm_global
from ._helpers_utils import wrap_entry
# 动态导入 global/index.py



# 注册 global 下所有导出到 VmSharedContext
for name in dir(vm_global):
    if name.startswith('_'):
        continue
    value = getattr(vm_global, name)
    VmSharedContext[name] = wrap_entry(name, value)



lib = vm_global
