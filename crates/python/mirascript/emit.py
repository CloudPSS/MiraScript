from typing import Any
from collections.abc import Callable

CommonContext: dict[str, Any] = dict()
"""MiraScript 通用执行上下文"""

CommonContext["debug_print"] = lambda message: print(f"Debug: {message}")


class Context(dict[str, Any]):
    """
    MiraScript 执行上下文
    """

    def __init__(self, **kwargs: Any) -> None:
        super().__init__(**kwargs)

    def __getitem__(self, key: str) -> Any:
        if key in self:
            return super().__getitem__(key)
        if key in CommonContext:
            return CommonContext[key]
        return None


Env: dict[str, Any] = dict()
"""MiraScript 执行环境"""

Env["Add"] = lambda x, y: x + y
Env["Context"] = Context
Env["Any"] = Any
Env["dict"] = dict
Uninitialized = type("Uninitialized", (), {})()
"""用于标记 MiraScript 中未初始化的变量"""
Env["Uninitialized"] = Uninitialized


def emit(bytecode: bytes) -> Callable[[Context], Any]:
    """
    生成 Python 函数

    Args:
        bytecode (str): 要生成的 Python 字节码

    Returns:
        (Callable[[Context], Any]): 生成的 Python 函数
    """

    # TODO: 解析 bytecode 并生成 Python 函数
    code = """
def script(context: Context | None = None) -> Any:
    if context is None:
        context = Context()
    context["debug_print"](Add(1, 2))
    """

    scope = {}
    exec(code, Env, scope)
    return scope["script"]
