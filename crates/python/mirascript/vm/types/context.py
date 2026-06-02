from typing_extensions import (
    Optional,
    Mapping,
    TypeAlias,
)

from .types import VmValue
from ...vm.error import VmError
from ...helpers.types import is_vm_value
from ...helpers.constants import kVmContext


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


VmContextLike: TypeAlias = Mapping[str, VmValue]


class VmContext(VmContextLike):
    """Mirascript 虚拟机上下文，提供全局变量访问接口"""

    def __init__(
        self,
        values: Optional[VmContextLike] = None,
        no_defaults: bool = False,
        **kwargs: VmValue,
    ):
        from .. import lib  # 加载全局库，确保全局库中的变量被注册到 VmSharedContext 中

        setattr(self, kVmContext, True)
        self._data = {} if no_defaults else dict(VmSharedContext)
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

    def __setitem__(self, key, value):
        _check_kv(key, value)
        self._data[key] = value

    def __iter__(self):
        return iter(self._data)

    def __len__(self):
        return len(self._data)

    def __contains__(self, key):
        return key in self._data

    def __repr__(self):
        return f"VmContext({self._data})"


# 全局共享上下文
VmSharedContext = VmContext(no_defaults=True)
