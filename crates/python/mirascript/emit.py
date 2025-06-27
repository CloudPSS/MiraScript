from typing import Any
from collections.abc import Callable


def emit(bytecode: str) -> Callable[[dict[str, Any]], Any]:
    """
    生成 Python 函数

    Args:
        bytecode (str): 要生成的 Python 字节码

    Returns:
        (Callable[[dict[str, Any]], Any]): 生成的 Python 函数
    """

    def _execute(context: dict[str, Any]) -> Any:
        # 在这里执行生成的 Python 字节码
        exec(bytecode, context)
        return context

    return _execute
