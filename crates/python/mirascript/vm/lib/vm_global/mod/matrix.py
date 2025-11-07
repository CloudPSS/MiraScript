from ..._helpers import (
    VmLib, expect_array, expect_callable, expect_const, required, throw_error,
    get_numbers, array_len, map_vm
)
from ....helpers import Cp
from ....operations import Add_, Call_, Div_, Mul_, Sub_, ToNumber_
from ....types import is_vm_any, is_vm_const, VmValue

def sizeImpl(matrix: VmValue):
    if not isinstance(matrix, list):
        return []
    if len(matrix) == 0:
        return [0]
    numRows = len(matrix)
    numCols = 0
    for row in matrix:
        if isinstance(row, list):
            numCols = max(numCols, len(row))
        else:
            return [numRows]
    return [numRows, numCols]

def num(v):
    return ToNumber_(v if v is not None else None)

x = lambda matrix: (
        required('matrix', matrix, []),
        sizeImpl(matrix)
    )[-1]

size = VmLib(
    lambda matrix: (
        required('matrix', matrix, []),
        sizeImpl(matrix)
    )[-1],
    {
        "summary": "获取矩阵尺寸",
        "params": {"matrix": "要获取尺寸的矩阵"},
        "paramsType": {"matrix": "any[][]"},
        "returnsType": "[number, number]",
    }
)

def transpose_impl(matrix):
    required('matrix', matrix, [])
    dims = sizeImpl(matrix)
    if len(dims) < 2:
        return matrix
    numRows, numCols = dims
    transposed = [[None for _ in range(numRows)] for _ in range(numCols)]
    for i in range(numRows):
        Cp()
        for j in range(numCols):
            row = matrix[i] if i < len(matrix) else None
            item = row[j] if row and j < len(row) else None
            transposed[j][i] = item
    return transposed

transpose = VmLib(
    transpose_impl,
    {
        "summary": "转置矩阵",
        "params": {"matrix": "要转置的矩阵"},
        "paramsType": {"matrix": "any[][]"},
        "returnsType": "any[][]",
    }
)

def entrywiseImpl(
    a, b, f,
    vvf=None, mmf=None, vmf=None, mvf=None
):
    dims_a = sizeImpl(a)
    dims_b = sizeImpl(b)
    ar, ac = (dims_a + [None, None])[:2]
    br, bc = (dims_b + [None, None])[:2]

    if ar is None:
        if br is None:
            return f(a, b)
        elif bc is None:
            return [f(a, b[r] if r < len(b) else None) for r in range(br)]
        else:
            return [
                [f(a, (b[r][c] if r < len(b) and c < len(b[r]) else None))
                 for c in range(bc)]
                for r in range(br)
            ]
    if br is None:
        if ac is None:
            return [f(a[r] if r < len(a) else None, b) for r in range(ar)]
        else:
            return [
                [f((a[r][c] if r < len(a) and c < len(a[r]) else None), b)
                 for c in range(ac)]
                for r in range(ar)
            ]
    if ac is None and bc is None:
        if vvf is not None:
            return vvf(a, b, ar, br)
        rr = max(ar, br)
        return [f(a[r] if r < len(a) else None, b[r] if r < len(b) else None) for r in range(rr)]

    if ac is None:
        if vmf is not None:
            return vmf(a, b, ar, br, bc)
        ac = ar
        ar = 1
        a = [a]
    if bc is None:
        if mvf is not None:
            return mvf(a, b, ar, ac, br)
        bc = br
        br = 1
        b = [b]
    if mmf is not None:
        return mmf(a, b, ar, ac, br, bc)
    rr = max(ar, br)
    rc = max(ac, bc)
    result = [[None for _ in range(rc)] for _ in range(rr)]
    for r in range(rr):
        for c in range(rc):
            aItem = a[0][0] if ar == 1 and ac == 1 else \
                (a[0][c] if ar == 1 else (a[r][0] if ac == 1 else a[r][c]))
            bItem = b[0][0] if br == 1 and bc == 1 else \
                (b[0][c] if br == 1 else (b[r][0] if bc == 1 else b[r][c]))
            result[r][c] = f(aItem, bItem)
    return result

entrywise = VmLib(
    lambda a, b, f: (
        expectConst('a', a, None),
        expectConst('b', b, None),
        expectCallable('f', f, None),
        entrywiseImpl(a, b, lambda a, b: (Cp(), Call_(f, [a, b]))[1] if isVmConst(Call_(f, [a, b])) else None)
    )[-1],
    {
        "summary": "逐项操作",
        "params": {"a": "第一个操作数", "b": "第二个操作数", "f": "操作函数"},
        "paramsType": {"a": "any | any[] | any[][]", "b": "any | any[] | any[][]", "f": "fn(a: any, b: any) -> any"},
        "returnsType": "any | any[] | any[][]",
    }
)

add = VmLib(
    lambda a, b: (
        expectConst('a', a, None),
        expectConst('b', b, None),
        entrywiseImpl(a, b, Add_)
    )[-1],
    {
        "summary": "逐项相加",
        "params": {"a": "第一个操作数", "b": "第二个操作数"},
        "paramsType": {"a": "number | number[] | number[][]", "b": "number | number[] | number[][]"},
        "returnsType": "number | number[] | number[][]",
    }
)

subtract = VmLib(
    lambda a, b: (
        expectConst('a', a, None),
        expectConst('b', b, None),
        entrywiseImpl(a, b, Sub_)
    )[-1],
    {
        "summary": "逐项相减",
        "params": {"a": "第一个操作数", "b": "第二个操作数"},
        "paramsType": {"a": "number | number[] | number[][]", "b": "number | number[] | number[][]"},
        "returnsType": "number | number[] | number[][]",
    }
)

entrywise_multiply = VmLib(
    lambda a, b: (
        expectConst('a', a, None),
        expectConst('b', b, None),
        entrywiseImpl(a, b, Mul_)
    )[-1],
    {
        "summary": "逐项相乘",
        "params": {"a": "第一个操作数", "b": "第二个操作数"},
        "paramsType": {"a": "number | number[] | number[][]", "b": "number | number[] | number[][]"},
        "returnsType": "number | number[] | number[][]",
    }
)

entrywise_divide = VmLib(
    lambda a, b: (
        expectConst('a', a, None),
        expectConst('b', b, None),
        entrywiseImpl(a, b, Div_)
    )[-1],
    {
        "summary": "逐项相除",
        "params": {"a": "第一个操作数", "b": "第二个操作数"},
        "paramsType": {"a": "number | number[] | number[][]", "b": "number | number[] | number[][]"},
        "returnsType": "number | number[] | number[][]",
    }
)

# multiply、invert、filled、zeros、ones、identity、diagonal 的实现同理，建议分块逐步转换。