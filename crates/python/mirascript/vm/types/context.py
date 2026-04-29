from typing_extensions import (
    Optional,
    Mapping,
    Dict,
)

from .types import VmValue
from ...vm.error import VmError
from ...helpers.types import is_vm_value
from ...helpers.constants import kVmContext

# 全局共享上下文
VmSharedContext: Dict[str, VmValue] = {}


class VmContext(Dict[str, VmValue]):
    """Mirascript 虚拟机上下文，提供全局变量访问接口"""

    def __init__(
        self,
        values: Optional[Mapping[str, VmValue]] = None,
        **kwargs: Dict[str, VmValue],
    ):
        setattr(self, kVmContext, True)
        self.update(VmSharedContext)
        if values:
            self._checked_merge(values)
        self._checked_merge(kwargs)

    def _checked_merge(self, other: Mapping[str, VmValue]):
        for key, value in other.items():
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
        self.update(other)

    def __getitem__(self, key):
        try:
            return super().__getitem__(key)
        except KeyError:
            raise VmError(f"Global variable '{key}' is not defined.", None)

    def __setitem__(self, key, value):
        raise VmError("Global variables are read-only and cannot be modified.", None)


# DefaultVmContext 实例
DefaultVmContext = VmContext()
