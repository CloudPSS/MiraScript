from __future__ import annotations
from typing_extensions import Optional, Mapping, TypeAlias, TYPE_CHECKING

from ..._helpers.types import is_vm_context, is_vm_value
from ..._helpers.constants import kVmContext
from ..error import VmError

if TYPE_CHECKING:
    from . import VmValue

    VmContextLike: TypeAlias = Mapping[str, VmValue]
else:
    VmContextLike = Mapping


def _check_kv(key, value):
    if not isinstance(key, str):
        raise VmError(
            f"Global variable name must be a string, got {type(key).__name__!r}",
            None,
        )
    if not is_vm_value(value):
        raise VmError(
            f"Invalid value for global variable '{key}': {value!r}",
            None,
        )


class VmContext(VmContextLike):
    """Mirascript 虚拟机上下文，提供全局变量访问接口"""

    def __init__(
        self,
        values: Optional[VmContextLike] = None,
        no_defaults: bool = False,
        **kwargs: VmValue,
    ):
        setattr(self, kVmContext, True)
        self._data = {} if no_defaults else dict(get_shared_context())
        if values:
            self._checked_merge(values)
        self._checked_merge(kwargs)

    def _checked_merge(self, other: VmContextLike):
        for key, value in other.items():
            _check_kv(key, value)
        self._data.update(other)

    def __getitem__(self, key):
        try:
            return self._data[key]
        except KeyError:
            raise VmError(f"Global variable '{key}' is not defined.", None) from None

    def __iter__(self):
        return iter(self._data)

    def __len__(self):
        return len(self._data)

    def __contains__(self, key):
        return key in self._data

    def __repr__(self):
        return f"VmContext({self._data})"


# 全局共享上下文
_shared_context: VmContext | None = None


def get_shared_context() -> VmContext:
    global _shared_context

    if _shared_context is None:
        # 注册全局变量到 VmSharedContext
        from ..lib._loader import register_globals

        context: dict[str, VmValue] = dict()
        register_globals(context)
        _shared_context = VmContext(values=context, no_defaults=True)
        assert is_vm_context(_shared_context)

    return _shared_context
