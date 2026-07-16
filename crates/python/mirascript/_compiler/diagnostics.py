from typing_extensions import TypeAlias, Literal

from .core import get_diagnostic_message

DiagnosticLevel: TypeAlias = Literal[
    "Error", "Warning", "Info", "Hint", "Reference", "SourceMap", "Unknown"
]


class DiagnosticPosition:
    """诊断位置"""

    start_line: int
    """起始行号"""
    start_column: int
    """起始列号"""
    end_line: int
    """结束行号"""
    end_column: int
    """结束列号"""

    def __init__(
        self,
        start_line: int,
        start_column: int,
        end_line: int,
        end_column: int,
    ):
        self.start_line = start_line
        self.start_column = start_column
        self.end_line = end_line
        self.end_column = end_column


class Diagnostic(DiagnosticPosition):
    """
    诊断信息类
    """

    __cache = {}

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
        super().__init__(start_line, start_column, end_line, end_column)
        self.code = code
        if code == 12000:
            self.level = "SourceMap"
            self.name = "SourceMap"
            self.message = "Source map information"
            return
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


class SourceMapEntry(DiagnosticPosition):
    """源映射信息"""


def decode_diagnostics(
    diagnostics: "list[int]",
) -> "tuple[list[Diagnostic], list[SourceMapEntry]]":
    """
    解析诊断信息

    Args:
        diagnostics (list[int]): 诊断信息列表，包含 [start_line, start_column, end_line, end_column, code]

    Returns:
        tuple[list[Diagnostic], list[SourceMapEntry]]: 解析后的诊断信息列表和源映射信息列表
    """
    diagnostics_list = []
    source_map_list = []
    for i in range(0, len(diagnostics), 5):
        code = diagnostics[i + 4]
        if code == 12000:
            source_map_list.append(
                SourceMapEntry(
                    start_line=diagnostics[i],
                    start_column=diagnostics[i + 1],
                    end_line=diagnostics[i + 2],
                    end_column=diagnostics[i + 3],
                )
            )
        else:
            diagnostics_list.append(
                Diagnostic(
                    start_line=diagnostics[i],
                    start_column=diagnostics[i + 1],
                    end_line=diagnostics[i + 2],
                    end_column=diagnostics[i + 3],
                    code=code,
                )
            )
    return diagnostics_list, source_map_list
