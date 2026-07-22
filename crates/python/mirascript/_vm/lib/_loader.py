from __future__ import annotations
from inspect import ismodule
from typing_extensions import cast, MutableMapping

from ..types import Uninitialized, VmValue
from . import vm_global
from ._helpers_utils import _wrap_entry


def register_globals(shared_context: MutableMapping[str, VmValue]):
    """将 global 下所有导出注册到 shared_context 中"""
    for name in dir(vm_global):
        if name.startswith("_") or name.endswith("_"):
            continue
        value = getattr(vm_global, name)
        if ismodule(value):
            continue
        if name == "Uninitialized" and value is Uninitialized:
            continue
        shared_context[name] = _wrap_entry(name, cast(VmValue, value), "global")
