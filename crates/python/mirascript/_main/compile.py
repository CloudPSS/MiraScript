from __future__ import annotations
import sys
import traceback

from .._compiler import compile as _compile, InputMode, VmScript, Diagnostic


def compile_code(
    code: str, mode: InputMode, filename: str | None = None
) -> tuple[VmScript | None, list[Diagnostic]]:
    try:
        script, diagnostics = _compile(code, input_mode=mode, filename=filename)
        return script, diagnostics
    except Exception as e:
        traceback.print_exc(file=sys.stderr)
        print(f"Error during compilation: {e}", file=sys.stderr)
        sys.exit(2)
