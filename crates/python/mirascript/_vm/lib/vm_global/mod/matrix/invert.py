from __future__ import annotations

from .....operations import Add, Div, Mul, Sub
from ...._helpers import _expect_const, _throw_error, _iterate
from ._helper import _size

__all__ = ["invert"]


def invert(matrix):
    _expect_const("matrix", matrix, [])
    dims = _size(matrix)

    if len(dims) == 0:
        return Div(1, matrix)
    if len(dims) == 1:
        return _iterate(
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
