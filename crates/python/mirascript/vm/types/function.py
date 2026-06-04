from dataclasses import dataclass
from inspect import signature
from typing_extensions import (
    TypeVar,
    ParamSpec,
    TypeAlias,
    Callable,
    TYPE_CHECKING,
    overload,
)

from ...helpers.constants import kVmFunction

if TYPE_CHECKING:
    from . import VmValue

P = ParamSpec("P")
V = TypeVar("V", bound="VmValue")
VmFunction: TypeAlias = Callable[P, "VmValue"]


@dataclass
class VmFunctionMeta:
    min_args: int = 0
    max_args: int = 0


_MAX_ARGS = 2**31 - 1


def _create_vm_function(name: str, fn: Callable) -> Callable:
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
def vm_function(name: str) -> Callable[[Callable[P, V]], Callable[P, V]]: ...
@overload
def vm_function(fn: Callable[P, V]) -> Callable[P, V]: ...
def vm_function(name_or_fn):  # type: ignore
    """将一个 Python 函数包装为一个 VmFunction"""
    if isinstance(name_or_fn, str):

        def decorator(fn: Callable[P, V], name=name_or_fn) -> Callable[P, V]:
            return _create_vm_function(name, fn)

        return decorator

    if not callable(name_or_fn):
        raise TypeError("Invalid function")

    return _create_vm_function(name_or_fn.__name__, name_or_fn)
