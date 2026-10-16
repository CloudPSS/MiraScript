from __future__ import annotations
import ast
from typing_extensions import TYPE_CHECKING

if TYPE_CHECKING:

    from .. import VmScript


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
    code = (
        ast.unparse(script.ast)
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
