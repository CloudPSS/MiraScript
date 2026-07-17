from __future__ import annotations
from typing_extensions import (
    TypeVar,
    ParamSpec,
    Callable,
    Protocol,
    TYPE_CHECKING,
    overload,
)
from dataclasses import dataclass
from inspect import signature

from ..._helpers.constants import kVmFunction

if TYPE_CHECKING:
    from . import VmValue
    from types import CodeType

P = ParamSpec("P")
V = TypeVar("V", bound="VmValue", covariant=True)


class VmFunction(Protocol[P, V]):
    """VmFunction 是一个可调用对象，它接受任意数量的参数，并返回一个 VmValue。"""

    def __call__(self, *args: P.args, **kwargs: P.kwargs) -> V: ...

    __name__: str
    __code__: CodeType


F = TypeVar("F", bound=VmFunction)


@dataclass
class VmFunctionMeta:
    min_args: int = 0
    max_args: int = 0


_MAX_ARGS = 2**31 - 1


def _create_vm_function(name: str, fn: F) -> F:
    info = VmFunctionMeta()
    sig = signature(fn)
    for param in sig.parameters.values():
        if param.kind is (param.VAR_POSITIONAL):
            info.max_args = _MAX_ARGS
        elif param.kind in (param.POSITIONAL_ONLY, param.POSITIONAL_OR_KEYWORD):
            info.max_args += 1
            if param.default is param.empty:
                info.min_args += 1
    fn.__name__ = name
    setattr(fn, kVmFunction, info)
    return fn


@overload
def vm_function(name: str) -> Callable[[F], F]: ...
@overload
def vm_function(fn: F) -> F: ...
def vm_function(name_or_fn):  # type: ignore
    """将一个 Python 函数包装为一个 VmFunction"""
    if isinstance(name_or_fn, str):

        def decorator(fn: F, name=name_or_fn) -> F:
            return _create_vm_function(name, fn)

        return decorator

    if not callable(name_or_fn):
        raise TypeError("Invalid function")

    return _create_vm_function(name_or_fn.__name__, name_or_fn)
