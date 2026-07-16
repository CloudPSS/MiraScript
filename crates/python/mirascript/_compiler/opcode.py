class OpCodes:
    """MiraScript 操作码"""

    def __init__(self) -> None:
        from .core import op_codes

        self.__dict__.update(op_codes())

    def __getattr__(self, name: str) -> int:
        if name in self.__dict__:
            return self.__dict__[name]
        raise AttributeError(f"No such OpCode: {name}")

    def __setattr__(self, name: str, value) -> None:
        raise AttributeError("OpCodes is immutable, cannot set attributes.")

    def __delattr__(self, name: str) -> None:
        raise AttributeError("OpCodes is immutable, cannot delete attributes.")


OpCode = OpCodes()
"""MiraScript 操作码"""

_ReverseOpCode = {v: k for k, v in OpCode.__dict__.items() if not k.startswith("_")}


def get_opcode_name(opcode: int) -> str:
    """获取操作码名称"""

    name = _ReverseOpCode.get(opcode, None)
    if name is None:
        return str(opcode)
    return name
