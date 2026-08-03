from __future__ import annotations
from typing_extensions import Callable, TYPE_CHECKING

if TYPE_CHECKING:

    from ast import AST

    from .. import VmScript


def _get_unparse() -> Callable[[AST], str]:
    import ast

    if hasattr(ast, "unparse"):
        return ast.unparse

    try:
        from astunparse import unparse  # pyright: ignore[reportMissingImports]

        return unparse
    except ImportError:
        raise ImportError(
            "Neither 'ast.unparse' nor 'astunparse' is available. Please install 'astunparse' with `pip install astunparse` to enable debug output generation."
        )


def _get_hint(script: VmScript) -> str:

    ext = "miratpl" if script.input_mode == "template" else "mira"
    return (
        '"""\nGenerated from '
        + script.filename.replace("\\", "/")
        + ":\n\n"
        + "`````"
        + ext
        + "\n"
        + script.source.rstrip("\r\n")
        + "\n"
        + "`````"
        + '\n"""\n'
    )


def unparse(script: VmScript, output_file: str, variables: dict) -> None:
    unparse = _get_unparse()
    code = (
        unparse(script.ast)
        if script.ast is not None
        else 'raise NotImplementedError("AST is not available")'
    )

    with open(output_file, "w", encoding="utf-8") as f:
        f.write(
            "# type: ignore\n"
            f"{_get_hint(script)}\n"
            f"{code}\n"
            "\n\n"
            "if __name__ == '__main__':\n"
            f"    result = script({variables})\n"
            "    from mirascript import display\n"
            "    print('[OK]', display(result))"
        )
