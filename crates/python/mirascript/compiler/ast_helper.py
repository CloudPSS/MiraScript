from ast import (
    AST,
    Assign,
    Call,
    Expr,
    Load,
    Name,
    Starred,
    Store,
    expr_context,
    expr,
    Subscript,
    iter_fields,
    Constant,
)
import sys
from typing_extensions import TypeVar, Sequence

T = TypeVar("T", bound=AST)


class ASTHelper:
    def __init__(
        self,
        lineno: int = 1,
        col_offset: int = 0,
        end_lineno: "int | None" = None,
        end_col_offset: "int | None" = None,
    ):
        self.lineno = lineno
        self.col_offset = col_offset
        self.end_lineno = end_lineno
        self.end_col_offset = end_col_offset

    def set_position(self, node: T, deep=False) -> T:
        """为AST节点设置行号和列偏移量"""

        if deep:
            for field, value in iter_fields(node):
                if isinstance(value, list):
                    for item in value:
                        if isinstance(item, AST):
                            self.set_position(item, deep=True)
                elif isinstance(value, AST):
                    self.set_position(value, deep=True)

        # 设置当前节点的位置信息
        if not hasattr(node, "lineno"):
            setattr(node, "lineno", self.lineno)
        if not hasattr(node, "col_offset"):
            setattr(node, "col_offset", self.col_offset)
        if self.end_lineno and not hasattr(node, "end_lineno"):
            setattr(node, "end_lineno", self.end_lineno)
        if self.end_col_offset and not hasattr(node, "end_col_offset"):
            setattr(node, "end_col_offset", self.end_col_offset)
        return node

    def var(self, name: str, ctx: expr_context = Load()) -> Name:
        """生成一个变量的 AST 节点"""
        return self.set_position(Name(id=name, ctx=ctx))

    def load_var(self, name: str) -> expr:
        """生成一个加载变量的 AST 节点"""
        if name == "None":
            return self.const(None)
        return self.var(name, ctx=Load())

    def store_var(self, name: str) -> Name:
        """生成一个存储变量的 AST 节点"""
        return self.var(name, ctx=Store())

    def const(self, value) -> Constant:
        """生成一个常量的 AST 节点"""
        return self.set_position(Constant(value=value))

    def expr(self, value: expr) -> Expr:
        """生成一个表达式语句的 AST 节点"""
        return self.set_position(Expr(value=value))

    def subscript(
        self, value: "str | expr", slice: expr, ctx: expr_context = Load()
    ) -> Subscript:
        """生成一个下标访问的 AST 节点"""

        # Compatibility: ast.Index was removed in Python 3.9.
        # Use ast.Index for older Pythons, and plain expr (ast.Constant) for 3.9+.
        if sys.version_info < (3, 9):
            slice = self.set_position(ast.Index(value=slice))

        if isinstance(value, str):
            value = self.load_var(value)

        return self.set_position(
            Subscript(
                value=value,
                slice=slice,
                ctx=ctx,
            )
        )

    def assign(self, target: "str | expr | Sequence[expr]", value: expr) -> Assign:
        """生成一个赋值语句的 AST 节点"""
        if isinstance(target, str):
            target = self.store_var(target)
        return self.set_position(
            Assign(
                targets=[target] if isinstance(target, expr) else list(target),
                value=value,
            )
        )

    def call(
        self, func: "expr | str", args: "Sequence[expr | str] | None" = None
    ) -> Call:
        """生成一个函数调用的 AST 节点"""
        if isinstance(func, str):
            func = self.load_var(func)
        if args is None:
            args = []
        return self.set_position(
            Call(
                func=func,
                args=[
                    self.load_var(arg) if isinstance(arg, str) else arg for arg in args
                ],
                keywords=[],
            )
        )

    def starred(self, value: "expr | str", ctx: expr_context = Load()) -> Starred:
        """生成一个星号表达式的 AST 节点"""
        if isinstance(value, str):
            value = self.load_var(value)
        return self.set_position(Starred(value=value, ctx=ctx))

    def assign_call(
        self,
        target: "str | expr | Sequence[expr]",
        func: "expr | str",
        args: "Sequence[expr | str] | None" = None,
    ) -> Assign:
        """生成一个赋值调用的 AST 节点"""
        call_node = self.call(func, args)
        return self.assign(target, call_node)

    def vm_element(
        self, args: "str | Sequence[str | expr]", helper_name="Element", spread=False
    ) -> "Call | Starred":
        """生成一个虚拟机元素的 AST 节点"""
        if isinstance(args, str):
            args = [args]
        call = self.call(helper_name, args)
        if spread:
            return self.starred(call)
        else:
            return call
