from pathlib import Path
import sys
import argparse
import traceback

from .vm.types.context import VmContext
from .compiler import compile, InputMode, VmScript, Diagnostic


def _compile(
    script: str, mode: InputMode, filename: "str | None" = None
) -> "tuple[VmScript | None, list[Diagnostic]]":
    try:
        bytecode, diagnostics = compile(script, input_mode=mode, filename=filename)
        return bytecode, diagnostics
    except Exception as e:
        traceback.print_exc()
        print(f"Error during compilation: {e}")
        sys.exit(2)


def _print_debug(script: VmScript, output_file):
    import ast

    unparse = getattr(ast, "unparse", None)
    if not unparse:
        try:
            import astunparse

            unparse = astunparse.unparse
        except ImportError:
            unparse = None
            raise ImportError(
                "Neither 'ast.unparse' nor 'astunparse' is available. Please install 'astunparse' with `pip install astunparse` to enable debug output generation."
            )

    with open(output_file, "w", encoding="utf-8") as f:
        f.write(
            f"{unparse(script.ast)}"
            "\n\n"
            "if __name__ == '__main__':\n"
            "    result = script()\n"
            "    print('[OK]', result)"
        )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Compile and execute a MiraScript file"
    )
    parser.add_argument(
        "-t",
        "--template",
        action="store_true",
        help="Indicates the input is a template file",
    )
    parser.add_argument(
        "-g",
        "--generate",
        action="store",
        help="Output generated code to the specified file, needs 'cli_debug' extra enabled",
    )
    parser.add_argument(
        "script_file",
        nargs="?",
        default="-",
        help="Path to the MiraScript file to compile (use '-' for stdin)",
    )
    args = parser.parse_args()
    if args.script_file == "-":
        script = sys.stdin.read()
        mode = "template" if args.template else "script"
        script_file = "<stdin>"
    else:
        script_file = Path(args.script_file).resolve()
        if not script_file.is_file():
            print(f"Error: File '{script_file}' does not exist.")
            sys.exit(1)
        script = script_file.read_text(encoding="utf-8")
        mode = (
            "template" if args.template or script_file.suffix != ".mira" else "script"
        )

    # Compile and execute the script
    result, diagnostics = _compile(script, mode, str(script_file))

    if result and args.generate:
        _print_debug(result, args.generate)

    for diag in diagnostics:
        print(diag)

    if result is None:
        sys.exit(1)

    assert callable(result), "Compilation failed, result is not callable"

    try:
        print("[OK]", result())
    except Exception as e:
        traceback.print_exc()
