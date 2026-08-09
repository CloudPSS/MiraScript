from __future__ import annotations
from enum import Enum
from typing_extensions import Callable, Mapping, TYPE_CHECKING, Final

from ..._helpers.types import is_vm_const, is_vm_context
from ..._helpers.constants import kVmScript
from ..types.context import VmContext, VmContextLike, get_shared_context
from ..types.module import VmModule
from ..types import VmAny, VmConst, VmRecord, VmValue
from ..types.function import VmFunction, vm_function
from .common import AssertInit
from .cp import CpEnter, CpExit

if TYPE_CHECKING:

    from ..._compiler.script import VmScript

__all__ = [
    "LoopContinue",
    "LoopBreak",
    "Script",
    "Module",
    "Pub",
    "Element",
    "ElementOpt",
    "Fn",
    "Upvalue",
]


class _LoopControl(Enum):
    Continue = object()
    Break = object()


LoopContinue: Final = _LoopControl.Continue
"""标记当前值未返回的值"""
LoopBreak: Final = _LoopControl.Break
"""标记当前值为Break"""


def Script(func: Callable[[VmContextLike], VmValue]) -> VmScript:

    def script_wrapper(context: VmContextLike | None = None):
        if context is None:
            context = get_shared_context()
        elif not is_vm_context(context):
            context = VmContext(context)

        try:
            CpEnter()
            return func(context)
        finally:
            CpExit()

    setattr(script_wrapper, kVmScript, True)
    return script_wrapper  # type: ignore


_PUB_ATTR = "__mirascript.mod.pub__"


class _Mod(Mapping[str, VmValue]):

    __slots__ = ("pub",)

    def __init__(self, pub: dict[str, Callable[[], VmAny]]):
        self.pub = pub

    def __getitem__(self, key: str) -> VmValue:
        getter = self.pub[key]
        return Upvalue(getter())

    def __iter__(self):
        return iter(self.pub.keys())

    def __len__(self):
        return len(self.pub)

    def __contains__(self, key) -> bool:
        return key in self.pub


def Module(name: str):

    def decorator(kls: type):

        pub: dict[str, Callable[[], VmAny]] = {}
        for attr_name in dir(kls):
            attr = getattr(kls, attr_name)
            pub_name = getattr(attr, _PUB_ATTR, None)
            if pub_name is not None:
                assert callable(attr), f"Public attribute {pub_name} must be callable"
                pub[pub_name] = attr  # pyright: ignore[reportArgumentType]

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
