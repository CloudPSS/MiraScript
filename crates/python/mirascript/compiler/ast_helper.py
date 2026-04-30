from ast import Assign, Call, Load, Name, Store, expr_context, expr, Subscript
import sys


class ASTHelper:
    def __init__(self, lineno: int = 1, col_offset: int = 0):
        self.lineno = lineno
        self.col_offset = col_offset

    def var(self, name: str, ctx: expr_context = Load()) -> Name:
        """生成一个变量的 AST 节点"""
        return Name(id=name, ctx=ctx, lineno=self.lineno, col_offset=self.col_offset)

    def load_var(self, name: str) -> Name:
        """生成一个加载变量的 AST 节点"""
        return self.var(name, ctx=Load())

    def store_var(self, name: str) -> Name:
        """生成一个存储变量的 AST 节点"""
        return self.var(name, ctx=Store())

    def subscript(
        self, value: "str | expr", slice: expr, ctx: expr_context = Load()
    ) -> Subscript:
        """生成一个下标访问的 AST 节点"""

        # Compatibility: ast.Index was removed in Python 3.9.
        # Use ast.Index for older Pythons, and plain expr (ast.Constant) for 3.9+.
        if sys.version_info < (3, 9):
            slice = ast.Index(
                value=slice, lineno=self.lineno, col_offset=self.col_offset
            )

        if isinstance(value, str):
            value = self.load_var(value)

        return Subscript(
            value=value,
            slice=slice,
            ctx=ctx,
            lineno=self.lineno,
            col_offset=self.col_offset,
        )

    def assign(self, target: "str | expr | list[expr]", value: expr) -> Assign:
        """生成一个赋值语句的 AST 节点"""
        if isinstance(target, str):
            target = self.store_var(target)
        return Assign(
            targets=[target] if isinstance(target, expr) else target,
            value=value,
            lineno=self.lineno,
            col_offset=self.col_offset,
        )

    def call(self, func: "expr | str", args: "list[expr | str] | None" = None) -> Call:
        """生成一个函数调用的 AST 节点"""
        if isinstance(func, str):
            func = self.load_var(func)
        if args is None:
            args = []
        return Call(
            func=func,
            args=[self.load_var(arg) if isinstance(arg, str) else arg for arg in args],
            keywords=[],
            lineno=self.lineno,
            col_offset=self.col_offset,
        )

    def assign_call(
        self,
        target: "str | expr | list[expr]",
        func: "expr | str",
        args: "list[expr | str] | None" = None,
    ) -> Assign:
        """生成一个赋值调用的 AST 节点"""
        call_node = self.call(func, args)
        return self.assign(target, call_node)
