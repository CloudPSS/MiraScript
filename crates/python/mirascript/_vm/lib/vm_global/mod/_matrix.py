import math

from ....._helpers.convert import to_number
from ....._helpers.types import is_vm_array, is_vm_const
from ....types.types import VmAny, VmConst, VmValue
from ....operations import Add, Call, Div, Mul, Sub, Cp
from ..._helpers import (
    _expect_array,
    _expect_callable,
    _expect_const,
    _expect_integer,
    _required,
    _throw_error,
    _get_numbers,
    _map_vm,
)
from ..._helpers_utils import _array_len

__all__ = [
    "size",
    "entrywise",
    "transpose",
    "add",
    "subtract",
    "entrywise_multiply",
    "entrywise_divide",
    "multiply",
    "invert",
    "diagonal",
    "zeros",
    "ones",
    "identity",
]


def _size_impl(matrix: VmAny) -> list[int]:
    if not is_vm_array(matrix):
        return []
    if len(matrix) == 0:
        return [0]

    num_rows = len(matrix)
    num_cols = 0

    for row in matrix:
        if is_vm_array(row):
            num_cols = max(num_cols, len(row))
        else:
            return [num_rows]
    return [num_rows, num_cols]


def _num(v: VmAny) -> float:
    return to_number(v)


def size(matrix: VmAny) -> list[float]:
    _required("matrix", matrix, [])
    return [float(n) for n in _size_impl(matrix)]


def transpose(matrix) -> VmAny:
    _required("matrix", matrix, [])
    dims = _size_impl(matrix)

    if len(dims) < 2:
        return matrix
    num_rows, num_cols = dims

    transpose = []

    for j in range(num_cols):
        Cp()
        newRow = []
        for i in range(num_rows):
            row = matrix[i] if i < len(matrix) else None
            item = row[j] if row and j < len(row) else None
            newRow.append(item)
        transpose.append(newRow)
    return transpose


def _entrywise_impl(a, b, f, vvf=None, mmf=None, vmf=None, mvf=None):
    aDims = _size_impl(a)
    bDims = _size_impl(b)

    if len(aDims) == 0:
        if len(bDims) == 0:
            return f(a, b)
        elif len(bDims) == 1:
            bLen = bDims[0]
            result = []
            for j in range(bLen):
                bItem = b[j] if j < len(b) else None
                result.append(f(a, bItem))
            return result
        else:
            numRows, numCols = bDims
            result = []
            for i in range(numRows):
                Cp()
                newRow = []
                for j in range(numCols):
                    row = b[i] if i < len(b) else None
                    bItem = row[j] if row and j < len(row) else None
                    newRow.append(f(a, bItem))
                result.append(newRow)
            return result

    if len(bDims) == 0:
        if len(aDims) == 1:
            aLen = aDims[0]
            result = []
            for i in range(aLen):
                aItem = a[i] if i < len(a) else None
                result.append(f(aItem, b))
            return result
        else:
            numRows, numCols = aDims
            result = []
            for i in range(numRows):
                Cp()
                newRow = []
                for j in range(numCols):
                    row = a[i] if i < len(a) else None
                    aItem = row[j] if row and j < len(row) else None
                    newRow.append(f(aItem, b))
                result.append(newRow)
            return result
    if len(aDims) == 1 and len(bDims) == 1:
        if vvf is not None:
            return vvf(a, b, aDims[0], bDims[0])
        aLen = aDims[0]
        bLen = bDims[0]
        result = []
        rr = max(aLen, bLen)
        for i in range(rr):
            aItem = a[i] if i < len(a) else None
            bItem = b[i] if i < len(b) else None
            result.append(f(aItem, bItem))
        return result

    if len(aDims) == 1:
        if vmf is not None:
            return vmf(a, b, aDims[0], bDims[0], bDims[1])
        aLen = aDims[0]
        numRows, numCols = bDims
        if aLen != numCols:
            _throw_error("Array length mismatch", [])
        result = []
        for i in range(numRows):
            newRow = []
            for j in range(numCols):
                aItem = a[j] if j < len(a) else None
                row = b[i] if i < len(b) else None
                bItem = row[j] if row and j < len(row) else None
                newRow.append(f(aItem, bItem))
            result.append(newRow)
        return result
    if len(bDims) == 1:
        if mvf is not None:
            return mvf(a, b, aDims[0], aDims[1], bDims[0])
        bLen = bDims[0]
        numRows, numCols = aDims
        if bLen != numCols:
            _throw_error("Array length mismatch", [])
        result = []
        for i in range(numRows):
            newRow = []
            for j in range(numCols):
                row = a[i] if i < len(a) else None
                aItem = row[j] if row and j < len(row) else None
                bItem = b[j] if j < len(b) else None
                newRow.append(f(aItem, bItem))
            result.append(newRow)
        return result
    if mmf is not None:
        return mmf(a, b, aDims[0], aDims[1], bDims[0], bDims[1])

    rr = max(aDims[0], bDims[0])
    cc = max(aDims[1], bDims[1])
    result = []
    for i in range(rr):
        newRow = []
        for j in range(cc):
            ar = 0 if aDims[0] == 1 else i
            ac = 0 if aDims[1] == 1 else j
            br = 0 if bDims[0] == 1 else i
            bc = 0 if bDims[1] == 1 else j
            aRow = a[ar] if ar < len(a) else None
            aItem = aRow[ac] if aRow and ac < len(aRow) else None
            bRow = b[br] if br < len(b) else None
            bItem = bRow[bc] if bRow and bc < len(bRow) else None
            newRow.append(f(aItem, bItem))

            # aRow = a[i] if  i < len(a) else None
            # aItem = aRow[j] if aRow and  j < len(aRow) else None
            # bRow = b[i] if i < len(b) else None
            # bItem = bRow[j] if bRow and  j < len(bRow) else None
            # newRow.append(f(aItem,bItem))
        result.append(newRow)
    return result


def entrywise(matrix, scalar, fn):
    _expect_const("matrix", matrix, [])
    _expect_const("scalar", scalar, [])
    _expect_callable("fn", fn, [])

    def f(a, b):
        ret = Call(fn, *(a, b))
        if not is_vm_const(ret):
            return None
        return ret

    return _entrywise_impl(matrix, scalar, f)


def add(a, b):
    _expect_const("a", a, [])
    _expect_const("b", b, [])
    return _entrywise_impl(a, b, Add)


def subtract(a, b):
    _expect_const("a", a, [])
    _expect_const("b", b, [])
    return _entrywise_impl(a, b, Sub)


def entrywise_multiply(a, b):
    _expect_const("a", a, [])
    _expect_const("b", b, [])
    return _entrywise_impl(a, b, Mul)


def entrywise_divide(a, b):
    _expect_const("a", a, [])
    _expect_const("b", b, [])
    return _entrywise_impl(a, b, Div)


def multiply(a, b):
    _expect_const("a", a, [])
    _expect_const("b", b, [])

    def vvf(a, b, aLen, bLen):
        rr = max(aLen, bLen)
        s = 0
        for i in range(rr):
            aItem = a[i] if i < len(a) else None
            bItem = b[i] if i < len(b) else None
            s += _num(aItem) * _num(bItem)

        return s

    def mmf(a, b, aRows, aCols, bRows, bCols):
        if aCols != bRows:
            _throw_error("Matrix size mismatch for multiplication", [])
        result = []
        for i in range(aRows):
            newRow = []
            for j in range(bCols):
                sum = 0
                for k in range(aCols):
                    aRow = a[i] if i < len(a) else None
                    aItem = aRow[k] if aRow and k < len(aRow) else None
                    bRow = b[k] if k < len(b) else None
                    bItem = bRow[j] if bRow and j < len(bRow) else None
                    sum = Add(sum, Mul(aItem, bItem))
                newRow.append(sum)
            result.append(newRow)
        return result

    def vmf(a, b, aLen, bRows, bCols):
        if aLen != bRows:
            _throw_error("Vector and matrix size mismatch for multiplication", [])

        result = []
        for i in range(bCols):
            item = 0
            for j in range(aLen):
                aItem = a[j] if j < len(a) else None
                bRow = b[j] if j < len(b) else None
                bItem = bRow[i] if bRow and i < len(bRow) else None
                # newRow.append(Mul_(aItem, bItem))
                item += _num(aItem) * _num(bItem)
            result.append(item)
        return result

    def mvf(a, b, aRows, aCols, bLen):
        if aCols != bLen:
            _throw_error("Matrix and vector size mismatch for multiplication", [])
        result = []
        for i in range(aRows):
            sum = 0
            for j in range(aCols):
                aRow = a[i] if i < len(a) else None
                aItem = aRow[j] if aRow and j < len(aRow) else None
                bItem = b[j] if j < len(b) else None
                sum = Add(sum, Mul(aItem, bItem))
            result.append(sum)
        return result

    return _entrywise_impl(a, b, Mul, vvf=vvf, mmf=mmf, vmf=vmf, mvf=mvf)


def invert(matrix):
    _expect_const("matrix", matrix, [])
    dims = _size_impl(matrix)

    if len(dims) == 0:
        return Div(1, matrix)
    if len(dims) == 1:
        return _map_vm(
            matrix,
            lambda *v: Div(1, v[0]),
        )

    numRows, numCols = dims
    if numRows != numCols:
        _throw_error("Only square matrices can be inverted", [])
    m = matrix

    if numRows == 1:
        return [[Div(1, m[0][0])]]
    if numRows == 2:
        det = Sub(Mul(m[0][0], m[1][1]), Mul(m[0][1], m[1][0]))
        if det == 0:
            _throw_error("Matrix is singular and cannot be inverted", [])
        return [
            [Div(m[1][1], det), Div(-m[0][1], det)],
            [Div(-m[1][0], det), Div(m[0][0], det)],
        ]

    A = []
    B = []

    for i in range(numRows):
        aRow = []
        bRow = []
        for j in range(numCols):
            aRow.append(m[i][j] if i < len(m) and j < len(m[i]) else None)
            bRow.append(1 if i == j else 0)
        A.append(aRow)
        B.append(bRow)
    for c in range(numCols):
        ABig = abs(A[c][c])
        rBig = c
        r = c + 1
        while r < numRows:
            if abs(A[r][c]) > ABig:
                ABig = abs(A[r][c])
                rBig = r
            r += 1

        r = rBig
        if r != c:
            A[c], A[r] = A[r], A[c]
            B[c], B[r] = B[r], B[c]
        AC = A[c]
        BC = B[c]

        for r in range(numRows):
            AR = A[r]
            BR = B[r]
            if r != c:
                if AR[c] == 0:
                    continue
                factor = Div(-AR[c], AC[c])
                for col in range(c, numCols):
                    AR[col] = Add(AR[col], Mul(factor, AC[col]))
                for col in range(numCols):
                    BR[col] = Add(BR[col], Mul(factor, BC[col]))
            else:
                factor = AC[c]
                for col in range(c, numCols):
                    AR[col] = Div(AR[col], factor)
                for col in range(numCols):
                    BR[col] = Div(BR[col], factor)
    return B


def diagonal(vector, k=0):
    _expect_array("vector", vector, [])
    fk = _expect_integer("k", k)
    if math.isnan(fk):
        fk = 0

    if all(is_vm_array(v) for v in vector):
        diag = []
        for i in range(len(vector)):
            row = vector[i]
            r = i + fk
            if r < 0:
                continue
            if not row or r >= len(row):
                continue
            diag.append(row[int(r)])

        return diag

    l = len(vector)
    m = _array_len(l - fk if fk < 0 else l)
    n = _array_len(l + fk if fk > 0 else l)

    result = []
    for i in range(m):
        newRow = []
        for j in range(n):
            if j - i == fk:
                ai = i if fk >= 0 else j
                vRow = vector[ai] if ai < len(vector) else None
                newRow.append(vRow)
            else:
                newRow.append(0)
        result.append(newRow)
    return result


def _filled(size: tuple[VmValue, ...], value: VmConst) -> VmConst:
    s = _get_numbers(size)
    if len(s) == 0:
        return []

    while len(s) > 0:
        repeat = _array_len(s.pop())
        Cp()
        # 从 MiraScript 语义而言，可以使用同一个引用
        data = [value] * repeat
        value = data
    return value


def zeros(*size):
    return _filled(size, 0)


def ones(*size):
    return _filled(size, 1)


def identity(*size):
    s = _get_numbers(size)
    if len(s) == 0:
        return []
    if len(s) > 2:
        _throw_error("Identity matrix must be 1D or 2D", [])
    if len(s) == 1:
        s = [s[0], s[0]]

    m = _array_len(s[0])
    n = _array_len(s[1])

    result = [[0.0] * n for _ in range(m)]

    for i in range(min(m, n)):
        result[i][i] = 1.0

    return result
