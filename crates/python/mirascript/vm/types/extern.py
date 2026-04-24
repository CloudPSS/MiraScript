# -*- coding: utf-8 -*-
from typing import Any

from mirascript.vm.error import VmError
from .wrapper import VmWrapper

# from .module import VmModule
# from ..error import VmError


def wrap_to_vm_value(value, caller=None):
    if value is None:
        return None
    if callable(value):
        return value
    if isinstance(value, VmWrapper):
        return value
    if isinstance(value, (str, int, float, bool)):
        return value
    # Python 没有 bigint，symbol，undefined
    return VmExtern(value, None)


def unwrap_from_vm_value(value):
    if value is None or not isinstance(value, VmExtern):
        return value
    if value.caller is None or not callable(value.value):
        return value.value
    caller = value.caller.value
    func = value.value

    def proxy(*args, **kwargs):
        return func(*args, **kwargs)

    return proxy


class VmExtern(VmWrapper):
    _VM_EXTERN_SYMBOL = "mirascript.vm.extern"

    def __init__(self, value, caller=None):
        super().__init__(value)
        self.caller = caller
        setattr(self, self._VM_EXTERN_SYMBOL, True)

    def access(self, key: str, read: bool) -> bool:
        if key.startswith("_"):
            return False
        if callable(self.value) and key in ("prototype", "arguments", "caller"):
            return False
        if hasattr(self.value, key):
            return True

        if not read:
            return True

        if not key in self.value:
            return False

        if key == "constructor":
            return False
        # 跳过内建属性
        return True

    def has(self, key: str) -> bool:
        return self.access(key, True)

    def get(self, key: str) -> Any:
        if not self.has(key):
            return None

        prop = self.value[key]
        return wrap_to_vm_value(prop, self)

    def set(self, key: str, value):
        if not self.access(key, False):
            return False
        prop = unwrap_from_vm_value(value)
        self.value[key] = prop
        return True

    def call(self, args):
        if not callable(self.value):
            raise VmError.from_("Not a callable extern", None, None)
        caller = self.caller.value if self.caller else None
        unwrapped_args = [unwrap_from_vm_value(a) for a in args]
        # Python 没有 Reflect.apply，直接调用
        ret = self.value(*unwrapped_args)
        return wrap_to_vm_value(ret, self)

    @property
    def callable(self) -> bool:
        return callable(self.value)

    def keys(self):
        keys = []
        for key, _ in self.value.items():
            if self.has(key):
                keys.append(key)
        return keys

    def same(self, other) -> bool:
        return (
            isinstance(other, VmExtern)
            and self.value is other.value
            and self.caller is other.caller
        )

    @property
    def type(self):
        return "extern"

    @property
    def describe(self) -> str:
        return type(self.value).__name__


def is_vm_extern(value) -> bool:
    if value is None or not isinstance(value, object):
        return False
    return hasattr(value, VmExtern._VM_EXTERN_SYMBOL)
