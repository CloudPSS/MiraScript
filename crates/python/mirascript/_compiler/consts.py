from struct import Struct
from typing_extensions import Tuple

from .._vm.types import VmPrimitive

_ORDINAL = Struct("<i")
_NUM = Struct("<d")
_LEN = Struct("<I")
_SLEN = Struct("<i")
_SBYTE = Struct("<b")


def _read_constant(data: bytes, offset: int) -> Tuple[VmPrimitive, int]:
    """解析常量"""

    type_byte = data[offset]
    if type_byte == 0:
        return None, 1
    elif type_byte == 1:
        return True, 1
    elif type_byte == 2:
        return False, 1
    elif type_byte == 3:
        (ordinal,) = _ORDINAL.unpack_from(data, offset + 1)
        return float(ordinal), 5
    elif type_byte == 4:
        (num,) = _NUM.unpack_from(data, offset + 1)
        return num, 9
    elif type_byte == 5:
        (length,) = _LEN.unpack_from(data, offset + 1)
        str_bytes = data[offset + 5 : offset + 5 + length]
        return str_bytes.decode("utf-8"), 5 + length
    else:
        raise ValueError(f"Invalid constant type: {type_byte}")


def read_constants(data: bytes) -> Tuple[VmPrimitive, ...]:
    """读取常量表"""

    constants = []
    offset = 0
    count = len(data)
    while offset < count:
        constant, size = _read_constant(data, offset)
        constants.append(constant)
        offset += size
    return tuple(constants)


def split_chunk(chunk: bytes) -> Tuple[bytes, bytes]:
    """分割 chunk 为常量表和代码"""

    if len(chunk) < 12:
        raise ValueError("Invalid chunk: too short to contain constant size")

    (chunk_size,) = _LEN.unpack_from(chunk, 0)
    if len(chunk) < chunk_size:
        raise ValueError("Invalid chunk: chunk size does not match actual size")
    (code_size,) = _LEN.unpack_from(chunk, 4)
    (const_size,) = _LEN.unpack_from(chunk, 8 + code_size)
    code_data = chunk[8 : 8 + code_size]
    const_data = chunk[12 + code_size : 12 + code_size + const_size]
    return const_data, code_data


def read_param(data: bytes, offset: int, wide: bool) -> Tuple[int, int]:
    """读取参数"""

    if wide:
        (param,) = _LEN.unpack_from(data, offset)
        return param, 4
    else:
        param = data[offset]
        return param, 1


def read_index(data: bytes, offset: int, wide: bool) -> Tuple[int, int]:
    """读取索引"""

    if wide:
        (index,) = _SLEN.unpack_from(data, offset)
        return index, 4
    else:
        (index,) = _SBYTE.unpack_from(data, offset)
        return index, 1
