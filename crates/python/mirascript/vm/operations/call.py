from ...helpers.types import is_vm_const
from ..error import VmError
from ..types.types import VmAny, VmValue, VmArray
from .common import AssertInit


def Call(func, *args) -> VmValue:
    for a in args:
        AssertInit(a)
    # if isinstance(func, VmExtern) and hasattr(func, "callable"):
    #     return func.call(args)
    if callable(func):
        if len(args) == 0:
            return func()
        return func(*args)
    raise VmError(f"{type(func)}, {func} object is not callable", None)


def Vargs(varags: "list[VmAny]") -> "VmArray":
    """将非常量元素置为 None，返回新列表"""
    return [var if is_vm_const(var) else None for var in varags]
