class OpCodes:
    """MiraScript 操作码"""

    def __init__(self) -> None:
        from ..mirascript import op_codes

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


def get_opcode_name(opcode: int) -> str:
    """获取操作码名称"""

    for attr_name in dir(OpCode):
        if not attr_name.startswith("_") and getattr(OpCode, attr_name, None) == opcode:
            return attr_name
    return str(opcode)
