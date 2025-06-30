
class OpCodes:
    """MiraScript 操作码"""

    def __init__(self) -> None:
        from .mirascript import op_codes

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

CommonContext= dict()
"""MiraScript 通用执行上下文"""

CommonContext["debug_print"] = lambda *message: print(f"[MiraScript] {message}")


class Context(dict):
    """
    MiraScript 执行上下文
    """

    def __init__(self, **kwargs) -> None:
        super().__init__(**kwargs)

    def __getitem__(self, key: str):
        if key in self:
            return super().__getitem__(key)
        if key in CommonContext:
            return CommonContext[key]
        return None


Env= dict()
"""MiraScript 执行环境"""

Env["Add"] = lambda x, y: x + y
Env["Context"] = Context
Env["dict"] = dict
Uninitialized = type("Uninitialized", (), {})()
"""用于标记 MiraScript 中未初始化的变量"""
Env["Uninitialized"] = Uninitialized


class Script(object):
    """
    MiraScript 生成的 Python 函数
    """

    def __call__(self, context=None) : ...


def emit(bytecode: bytes) -> Script:
    """
    生成 Python 函数

    Args:
        bytecode (str): 要生成的 Python 字节码

    Returns:
        (Callable[[Context | None], Any]): 生成的 Python 函数
    """

    # TODO: 解析 bytecode 并生成 Python 函数
    code = """
def script(context=None):
    if context is None:
        context = Context()
    context["debug_print"](Add(1, 2), 2, 3)
    """

    scope = {}
    exec(code, Env, scope)
    return scope["script"]
