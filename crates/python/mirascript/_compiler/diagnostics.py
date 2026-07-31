from __future__ import annotations
from typing_extensions import TypeAlias, Literal, ClassVar
from dataclasses import dataclass, field

DiagnosticLevel: TypeAlias = Literal[
    "Error", "Warning", "Info", "Hint", "Reference", "SourceMap", "Unknown"
]


@dataclass(frozen=True)
class DiagnosticPosition:
    """诊断位置"""

    start_line: int = field()
    """起始行号"""
    start_column: int = field()
    """起始列号"""
    end_line: int = field()
    """结束行号"""
    end_column: int = field()
    """结束列号"""


@dataclass(frozen=True)
class Diagnostic(DiagnosticPosition):
    """
    诊断信息类
    """

    _cache: ClassVar[dict[int, tuple[str, str, str]]] = {}

    code: int = field()
    """诊断代码"""
    level: DiagnosticLevel = field(init=False, compare=False)
    """诊断级别"""
    name: str = field(init=False, compare=False)
    """诊断名称"""
    message: str = field(init=False, compare=False)
    """诊断消息"""

    def __post_init__(self):
        if self.code == 12000:
            object.__setattr__(self, "level", "SourceMap")
            object.__setattr__(self, "name", "SourceMap")
            object.__setattr__(self, "message", "Source map information")
            return
        info = Diagnostic._cache.get(self.code)
        if info is None:
            try:
                from .core import get_diagnostic_message

                info = get_diagnostic_message(self.code)
                Diagnostic._cache[self.code] = info
            except Exception:
                info = (
                    "Unknown",
                    f"{self.code}",
                    f"Unknown diagnostic code",
                )
            Diagnostic._cache[self.code] = info
        object.__setattr__(self, "level", info[0])
        object.__setattr__(self, "name", info[1])
        object.__setattr__(self, "message", info[2])

    def __repr__(self) -> str:
        start = f"({self.start_line}, {self.start_column})"
        end = f"({self.end_line}, {self.end_column})"
        return f"Diagnostic(code={self.code}, level={self.level}, name={self.name}, start={start}, end={end})"

    def __str__(self) -> str:
        return (
            f"[{self.level}] {self.message} ({self.name}) at "
            f"{self.start_line}:{self.start_column} - "
            f"{self.end_line}:{self.end_column}"
        )


@dataclass(frozen=True)
class SourceMapEntry(DiagnosticPosition):
    """源映射信息"""


def decode_diagnostics(
    diagnostics: list[int],
) -> tuple[list[Diagnostic], list[SourceMapEntry]]:
    """
    解析诊断信息

    Args:
        diagnostics (list[int]): 诊断信息列表，包含 [start_line, start_column, end_line, end_column, code]

    Returns:
        tuple[list[Diagnostic], list[SourceMapEntry]]: 解析后的诊断信息列表和源映射信息列表
    """
    diagnostics_list: list[Diagnostic] = []
    source_map_list: list[SourceMapEntry] = []
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
