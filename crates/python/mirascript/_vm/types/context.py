from __future__ import annotations
from typing_extensions import (
    Mapping,
    TypeAlias,
    MutableMapping,
    TYPE_CHECKING,
)

from ..._helpers.types import is_vm_value
from ..._helpers.constants import kVmContext, Uninitialized
from ..error import VmError

if TYPE_CHECKING:
    from . import VmValue, VmAny

    VmContextLike: TypeAlias = Mapping[str, VmValue]
else:
    VmContextLike = Mapping


def _get(dict: VmContextLike, key: str) -> VmAny:
    try:
        value = dict[key]
    except KeyError:
        return Uninitialized
    if type(value) is int:
        return float(value)
    if not is_vm_value(value):
        raise TypeError(f"Invalid value for global variable '{key}': {value!r}")
    return value


class VmContext(VmContextLike):
    """Mirascript 虚拟机上下文，提供全局变量访问接口"""

    __slots__ = (kVmContext, "data", "no_defaults")

    def __init__(
        self,
        data: VmContextLike | None = None,
        no_defaults: bool = False,
        # /,
        **kwargs: VmValue,
    ):

        setattr(self, kVmContext, True)
        self.data: VmContextLike = {} if data is None else data
        self.no_defaults = no_defaults

        if len(kwargs) > 0:
            if not isinstance(self.data, MutableMapping):
                raise TypeError(
                    "Cannot set global variables in non-mutable context with initial values."
                )
            for key in kwargs:
                value = _get(kwargs, key)
                if value is Uninitialized:
                    continue
                self.data[key] = value

    def __getitem__(self, key: str) -> VmValue:
        own = _get(self.data, key)
        if own is not Uninitialized:
            return own
        if self.no_defaults:
            raise VmError(f"Global variable '{key}' is not defined.", None) from None
        global_context = get_shared_context()
        return global_context[key]

    def __iter__(self):
        return iter(self.data)

    def __len__(self):
        return len(self.data)

    def __contains__(self, key: object) -> bool:
        if not isinstance(key, str):
            return False
        if key in self.data:
            return True
        if self.no_defaults:
            return False
        return key in get_shared_context()

    def __repr__(self):
        return f"VmContext({self.data})"


# 全局共享上下文
_shared_context: VmContext | None = None


def get_shared_context() -> VmContext:
    global _shared_context

    if _shared_context is None:
        # 注册全局变量到 VmSharedContext
        from ..lib._loader import register_globals

        context: dict[str, VmValue] = dict()
        register_globals(context)
        _shared_context = VmContext(context, True)

    return _shared_context
