from __future__ import annotations
from enum import Enum
from typing_extensions import Callable, Mapping, Any

from ..._helpers.types import is_vm_const, is_vm_context
from ..._helpers.constants import kVmScript
from ..types.context import VmContext, VmContextLike, get_shared_context
from ..types.module import VmModule
from ..types.types import VmAny, VmConst, VmRecord, VmValue
from ..types.function import VmFunction, vm_function
from .common import AssertInit
from .cp import CpEnter, CpExit


class _LoopControl(Enum):
    Continue = object()
    Break = object()


LoopContinue = _LoopControl.Continue
"""标记当前值未返回的值"""
LoopBreak = _LoopControl.Break
"""标记当前值为Break"""


def Script(func: Callable[..., Any]) -> Callable[..., Any]:

    def script_wrapper(*args, **kwargs):
        try:
            CpEnter()
            return func(*args, **kwargs)
        finally:
            CpExit()

    setattr(script_wrapper, kVmScript, True)
    return script_wrapper


_PUB_ATTR = "__mirascript.mod.pub__"


class _Mod(Mapping[str, VmValue]):
    def __init__(self, pub: dict[str, Callable[[], Any]]):
        self._pub = pub

    def __getitem__(self, key: str) -> VmValue:
        getter = self._pub[key]
        return Upvalue(getter())

    def __iter__(self):
        return iter(self._pub.keys())

    def __len__(self):
        return len(self._pub)

    def __contains__(self, key) -> bool:
        return key in self._pub


def Module(name: str):

    def decorator(kls: type):

        pub: dict[str, Callable[[], Any]] = {}
        for attr_name in dir(kls):
            attr = getattr(kls, attr_name)
            pub_name = getattr(attr, _PUB_ATTR, None)
            if pub_name is not None:
                assert callable(attr), f"Public attribute {pub_name} must be callable"
                pub[pub_name] = attr

        return VmModule(name, _Mod(pub))

    return decorator


def Pub(name: str):

    def decorator(method: Callable):
        assert callable(method), f"Public attribute {name} must be callable"
        setattr(method, _PUB_ATTR, name)
        return method

    return decorator


def Element(value: VmAny) -> VmConst | None:
    AssertInit(value)
    return value if is_vm_const(value) else None


def ElementOpt(key: str, value: VmAny) -> VmRecord:
    AssertInit(value)
    if value is None or not is_vm_const(value):
        return {}
    return {key: value}


def Fn(name: str) -> Callable[[VmFunction], VmFunction]:

    def decorator(func: VmFunction):

        assert callable(func), f"Function {name} must be callable"

        def fn_wrapper(*args, **kwargs):
            try:
                CpEnter()
                return func(*args, **kwargs)
            finally:
                CpExit()

        if hasattr(func.__code__, "replace"):
            func.__code__ = func.__code__.replace(co_name=name)

        return vm_function(name)(fn_wrapper)

    return decorator


def Upvalue(value: VmAny) -> VmValue:
    AssertInit(value)
    return value  # type: ignore


def Context(context: VmContextLike | None = None) -> VmContext:
    if context is None:
        return get_shared_context()
    if not is_vm_context(context):
        return VmContext(context)
    return context
