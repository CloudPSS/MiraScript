from typing import Any
from collections.abc import Callable
from .mirascript import Config
from .emit import emit
from .diagnostics import decode_diagnostics, CompilationError


def compile(script: str, config: Config) -> Callable[[dict[str, Any]], Any]:
    """
    编译 MiraScript 代码，生成 Python 函数

    Args:
        script (str): 要编译的 MiraScript 代码
        config (Config): 编译配置

    Returns:
        (Callable[[dict[str, Any]], Any]): 编译后的 Python 函数
    """
    bytecode, diagnostics = mirascript.compile(script, config)

    if bytecode is None:
        diagnostics = decode_diagnostics(diagnostics)
        raise CompilationError(diagnostics)
    return emit(bytecode)


# result = mirascript.compile("print('Hello, world!')", mirascript.Config({}))
# print(result)
