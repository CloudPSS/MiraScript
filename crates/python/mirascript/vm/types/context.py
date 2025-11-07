# -*- coding: utf-8 -*-
from typing import Any, Callable, Dict, Iterable, Optional
from collections.abc import Mapping
from .checker import is_vm_any

# from .function import VmFunction
from .extern import wrap_to_vm_value

# 依赖的外部函数和类型需在其他模块实现或导入
# from . import wrap_to_vm_value, is_vm_any

# 全局共享上下文
VmSharedContext: Dict[str, Any] = {}

# def define_vm_context_value(name: str, value: Any, override: bool = False):
# 	if not override and name in VmSharedContext:
# 		raise RuntimeError(f"Global variable '{name}' is already defined.")
# 	v = value
# 	# 如果是函数且不是VmFunction，自动包裹
# 	if callable(value) :
# 		v = VmFunction(value, isLib=True, fullName=f"global.{name}")
# 	VmSharedContext[name] = v if v is not None else None

_K_VM_CONTEXT = "__mira_vm_context__"


class VmContext:
    def __init__(self):
        setattr(self, _K_VM_CONTEXT, True)

    def keys(self) -> Iterable[str]:
        return VmSharedContext.keys()

    def get(self, key: str) -> Any:
        return VmSharedContext.get(key, None)

    def has(self, key: str) -> bool:
        return key in VmSharedContext

    def __getattr__(self, key):
        return self.get(key)

    def __getitem__(self, key):
        return self.get(key)


class ValueVmContext(VmContext):
    def __init__(self, env: Dict[str, Any]):
        super().__init__()
        self.env = env
        self._cached_keys = None

    def keys(self) -> Iterable[str]:
        if self._cached_keys is None:
            self._cached_keys = list(self.env.keys())
        return list(self._cached_keys) + list(VmSharedContext.keys())

    def get(self, key: str) -> Any:
        return self.env.get(key, None)

    def has(self, key: str) -> bool:
        return key in self.env


class FactoryVmContext(VmContext):
    def __init__(
        self,
        getter: Callable[[str], Any],
        enumerator: Optional[Callable[[], Iterable[str]]] = None,
    ):
        super().__init__()
        self.getter = getter
        self.enumerator = enumerator

    def keys(self) -> Iterable[str]:
        if not self.enumerator:
            return VmSharedContext.keys()
        return list(self.enumerator()) + list(VmSharedContext.keys())

    def get(self, key: str) -> Any:
        value = self.getter(key)
        if value is not None:
            return value
        return VmSharedContext.get(key, None)

    def has(self, key: str) -> bool:
        return self.getter(key) is not None or key in VmSharedContext


def create_vm_context(*args) -> VmContext:
    if len(args) == 0 or (args[0] is None and (len(args) < 2 or args[1] is None)):
        return VmContext()
    if callable(args[0]):
        return FactoryVmContext(args[0], args[1] if len(args) > 1 else None)
    
    
    vm_values = args[0] if len(args) > 0 else None
    extern_values = args[1] if len(args) > 1 else None
    env = dict(VmSharedContext)
    if vm_values:
        for key, value in vm_values.items():
            if not is_vm_any(value, False):
                continue
            env[key] = value if value is not None else None
    if extern_values:
        for key, value in extern_values.items():
            env[key] = None if value is None else wrap_to_vm_value(value, None)
    return ValueVmContext(env)


def is_vm_context(context: Any) -> bool:
    return (
        isinstance(context, VmContext)
        and getattr(context, _K_VM_CONTEXT, False) is True
    )


# DefaultVmContext 实例
DefaultVmContext = VmContext()
