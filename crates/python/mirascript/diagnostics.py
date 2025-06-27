from typing import Literal, TypedDict
from .mirascript import get_diagnostic_message


DiagnosticLevel = Literal["Error", "Warning", "Info", "Hint", "Reference", "Unknown"]


class Diagnostic:
    """
    诊断信息类
    """

    __cache: dict[int, tuple[DiagnosticLevel, str, str]] = {}

    start_line: int
    """起始行号"""
    start_column: int
    """起始列号"""
    end_line: int
    """结束行号"""
    end_column: int
    """结束列号"""
    code: int
    """诊断代码"""
    level: DiagnosticLevel
    """诊断级别"""
    name: str
    """诊断名称"""
    message: str
    """诊断消息"""

    def __init__(
        self,
        start_line: int,
        start_column: int,
        end_line: int,
        end_column: int,
        code: int,
    ):
        self.start_line = start_line
        self.start_column = start_column
        self.end_line = end_line
        self.end_column = end_column
        self.code = code
        info = Diagnostic.__cache.get(self.code)
        if info is None:
            try:
                info = get_diagnostic_message(self.code)
                Diagnostic.__cache[self.code] = info
            except:
                info = (
                    "Unknown",
                    f"{self.code}",
                    f"Unknown diagnostic code",
                )
            Diagnostic.__cache[self.code] = info
        self.level = info[0]
        self.name = info[1]
        self.message = info[2]

    def __repr__(self) -> str:
        return f"Diagnostic(code={self.code}, level={self.level}, name={self.name}, start=({self.start_line}, {self.start_column}), end=({self.end_line}, {self.end_column}))"

    def __str__(self) -> str:
        return (
            f"[{self.level}] {self.message} ({self.name}) at "
            f"{self.start_line}:{self.start_column} - "
            f"{self.end_line}:{self.end_column}"
        )


class CompilationError(Exception):
    """
    编译错误异常

    Attributes:
        diagnostics (list[Diagnostic]): 诊断信息列表
    """

    def __init__(self, diagnostics: list[Diagnostic]):
        super().__init__("Compilation failed")
        self.diagnostics = diagnostics

    def __repr__(self) -> str:
        return f"CompilationError({self.diagnostics})"

    def __str__(self) -> str:
        return "\n" + "\n".join(f"  {diag}" for diag in self.diagnostics)


def decode_diagnostics(diagnostics: list[int]) -> list[Diagnostic]:
    """
    解析诊断信息

    Args:
        diagnostics (list[int]): 诊断信息列表，包含 [start_line, start_column, end_line, end_column, code]

    Returns:
        list[Diagnostic]: 解析后的诊断信息列表
    """
    return [Diagnostic(*diagnostics[i : i + 5]) for i in range(0, len(diagnostics), 5)]
