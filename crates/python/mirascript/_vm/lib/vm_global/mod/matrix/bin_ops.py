from __future__ import annotations

from ......_helpers.types import is_vm_const
from .....operations import Add, Call, Div, Mul, Sub, Cp
from ...._helpers import _expect_callable, _expect_const, _throw_error
from ._helper import _size, _num

__all__ = [
    "entrywise",
    "add",
    "subtract",
    "entrywise_multiply",
    "entrywise_divide",
    "multiply",
]


def _entrywise_impl(a, b, f, vvf=None, mmf=None, vmf=None, mvf=None):
    aDims = _size(a)
    bDims = _size(b)

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
        ret = Call(fn, a, b)
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
