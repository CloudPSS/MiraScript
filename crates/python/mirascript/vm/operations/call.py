from ...helpers.types import is_vm_const, is_vm_function
from ...helpers.constants import kVmFunction
from ..error import VmError
from ..types.types import VmAny, VmValue, VmArray
from .common import AssertInit


def Call(func: VmAny, *args: VmAny) -> VmValue:
    for a in args:
        AssertInit(a)
    if is_vm_function(func):
        info = getattr(func, kVmFunction)
        arg_count = len(args)
        if info.min_args <= arg_count <= info.max_args:
            return func(*args)
        if arg_count < info.min_args:
            args = args + (None,) * (info.min_args - arg_count)
            return func(*args)
        if arg_count > info.max_args:
            args = args[: info.max_args]
            return func(*args)
    raise VmError(f"{type(func)}, {func} object is not callable", None)


def Vargs(varags: "list[VmAny]") -> "VmArray":
    """将非常量元素置为 None，返回新列表"""
    return [var if is_vm_const(var) else None for var in varags]
