from __future__ import annotations
from inspect import ismodule
from typing_extensions import cast, MutableMapping

from ..types.types import Uninitialized, VmValue
from ..._helpers import types as checker
from ..._helpers import convert as converter
from .. import types
from . import vm_global
from ._helpers_utils import wrap_entry


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
        if hasattr(types, name) and getattr(types, name) == value:
            continue
        if hasattr(checker, name) and getattr(checker, name) == value:
            continue
        if hasattr(converter, name) and getattr(converter, name) == value:
            continue
        shared_context[name] = wrap_entry(name, cast(VmValue, value), "global")
