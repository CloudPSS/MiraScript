from typing import Any
from collections.abc import Callable
from . import mirascript
from .emit import emit

Config = mirascript.Config


def compile(script: str, **config: Config) -> Callable[[dict[str, Any]], Any]:
    """
    编译 MiraScript 代码，生成 Python 函数

    Args:
        script (str): 要编译的 MiraScript 代码
        **config (Config): 编译配置，见 `help(mirascript.Config)`

    Returns:
        (Callable[[dict[str, Any]], Any]): 编译后的 Python 函数
    """
    cfg = Config(**config)
    bytecode, diagnostics = mirascript.compile(script, cfg)
    return emit(bytecode)


# result = mirascript.compile("print('Hello, world!')", mirascript.Config({}))
# print(result)
