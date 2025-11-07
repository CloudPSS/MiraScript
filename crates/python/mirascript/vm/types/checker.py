# -*- coding: utf-8 -*-
import math
from typing import Any

from .const import VmFunctionWrapper
from .wrapper import VmWrapper
from .module import VmModule


MAX_DEPTH = 100




def is_vm_primitive(value) -> bool:
    return isinstance(value, (str, int, float, bool)) or value is None


def is_vm_array(value):
    # VmArray 应为普通数组
    if not isinstance(value, list):
        return False
    return True


def is_vm_record(value):
    # VmRecord 应为普通对象或空原型对象
    if not isinstance(value, dict):
        return False
    return True


def is_vm_const_inner(value, depth):
    if depth >= MAX_DEPTH:
        return False
    depth += 1
    if isinstance(value, (str, int, float, bool)) or value is None:
        return True
    if isinstance(value, VmWrapper):
        return False
    if isinstance(value, list):
        return is_vm_array(value)
    if isinstance(value, dict):
        return is_vm_record(value)
    return False


def is_vm_const(value, check_deep=False):
    if isinstance(value, (str, int, float, bool)):
        return True
    if value is None:
        return True
    if isinstance(value, VmWrapper):
        return False
    if not check_deep:
        if isinstance(value, list):
            return is_vm_array(value)
        if isinstance(value, dict):
            return is_vm_record(value)
        return False
    else:
        return is_vm_const_inner(value, 1)


def is_vm_immutable(value, check_deep=False):
    # VmModule 需补充
    return False  #


def is_vm_value(value, check_deep=False):
    if value is None:
        return False
    return is_vm_any(value, check_deep)


def is_vm_any(value, check_deep=False):
    if isinstance(value, (str, int, float, bool)) or value is None:
        return True
    if isinstance(value, VmWrapper):
        return True
    if isinstance(value, list) or isinstance(value, dict):
        return is_vm_const(value, check_deep)
    if callable(value):
        return True
    return False


def is_vm_module(value) -> bool:
    # VmModule 需补充
    return isinstance(value, VmModule)


## 判断正负零
def is_positive_zero(value: Any) -> bool:
    return math.copysign(1.0, value) > 0 and value == 0.0


def is_negative_zero(value: Any) -> bool:
    return math.copysign(1.0, value) < 0 and value == 0.0
