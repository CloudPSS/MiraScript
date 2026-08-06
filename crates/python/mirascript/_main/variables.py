from __future__ import annotations
import sys

from .compile import compile_code
from .. import VmValue


def _parse_variable_definition(var: str) -> tuple[str, VmValue] | None:
    if "=" not in var:
        print(
            f"Error: Invalid variable definition '{var}'. Expected format NAME=VALUE.",
            file=sys.stderr,
        )
        return None
    name, value = var.split("=", 1)
    try:
        script, diagnostics = compile_code(
            f"return ({value});", "script", f"<variable:{name}>"
        )
        if script is None:
            print(
                f"Error: Failed to compile variable '{name}={value}'. Diagnostics:",
                *[diag for diag in diagnostics if diag.level == "Error"],
                file=sys.stderr,
            )
            return None
        return name, script()
    except Exception as e:
        print(f"Error evaluating variable '{name}={value}': {e}", file=sys.stderr)
        return None


def parse_variables(variable_list: list[str] | None) -> dict[str, VmValue] | None:
    if variable_list is None:
        return {}
    variables: dict[str, VmValue] = {}
    has_error = False
    for var in variable_list:
        result = _parse_variable_definition(var)
        if result is None:
            has_error = True
            continue
        name, value = result
        variables[name] = value
    if has_error:
        return None
    return variables
