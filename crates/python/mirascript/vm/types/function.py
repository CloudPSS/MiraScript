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


@overload
def vm_function(name: str) -> Callable[[Callable[P, V]], Callable[P, V]]: ...
@overload
def vm_function(fn: Callable[P, V]) -> Callable[P, V]: ...
def vm_function(name_or_fn):  # type: ignore
    """将一个 Python 函数标记为一个 VmFunction"""
    if isinstance(name_or_fn, str):

        def decorator(fn: Callable[P, V], name=name_or_fn) -> Callable[P, V]:
            fn.__name__ = name
            setattr(fn, kVmFunction, True)
            return fn

        return decorator
    if not callable(name_or_fn):
        raise TypeError("Invalid function")

    setattr(name_or_fn, kVmFunction, True)
    return name_or_fn
