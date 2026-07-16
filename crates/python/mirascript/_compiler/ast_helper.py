from ast import (
    AST,
    Assign,
    AugAssign,
    BinOp,
    Call,
    Compare,
    Del,
    Dict,
    Expr,
    For,
    FunctionDef,
    If,
    List,
    Load,
    Name,
    Pass,
    Return,
    Starred,
    Store,
    Tuple,
    While,
    arg,
    arguments,
    cmpop,
    expr_context,
    expr,
    Subscript,
    iter_fields,
    Constant,
    operator,
    stmt,
    boolop,
    unaryop,
)
import ast
import sys
from typing_extensions import (
    TypeVar,
    Iterable,
    Sequence,
    Literal,
    TypeAlias,
    assert_never,
    TYPE_CHECKING,
)

from .._helpers.checker import is_number

if TYPE_CHECKING:

    from .._vm.types.types import VmPrimitive

    T = TypeVar("T", bound=AST)
    Ctx: TypeAlias = Literal["Load", "Store", "Del", None]
    Op: TypeAlias = Literal[
        "Add",
        "Sub",
        "Mult",
        "MatMult",
        "Div",
        "Mod",
        "Pow",
        "LShift",
        "RShift",
        "BitOr",
        "BitXor",
        "BitAnd",
        "FloorDiv",
    ]
    CmpOp: TypeAlias = Literal[
        "Eq",
        "NotEq",
        "Lt",
        "LtE",
        "Gt",
        "GtE",
        "Is",
        "IsNot",
        "In",
        "NotIn",
    ]
    BoolOp: TypeAlias = Literal["And", "Or"]
    UnaryOp: TypeAlias = Literal["Invert", "Not", "UAdd", "USub"]


class ASTHelper:
    def __init__(
        self,
        lineno: int = 0,
        col_offset: int = 0,
        end_lineno: "int | None" = None,
        end_col_offset: "int | None" = None,
        source_lines: "Sequence[str] | None" = None,
    ):
        self.lineno = lineno
        self.col_offset = col_offset
        self.end_lineno = end_lineno
        self.end_col_offset = end_col_offset
        self.source_lines = source_lines

    def ctx(self, ctx: "Ctx" = None) -> expr_context:
        if ctx == "Load" or ctx is None:
            return self.set_position(Load())
        elif ctx == "Store":
            return self.set_position(Store())
        elif ctx == "Del":
            return self.set_position(Del())
        else:
            assert_never(ctx)
            raise ValueError(f"Invalid context: {ctx}")

    def op(self, op: "Op") -> operator:
        kls = getattr(ast, op, None)
        if kls is None or not issubclass(kls, operator):
            raise ValueError(f"Invalid operator: {op}")
        return self.set_position(kls())

    def cmpop(self, op: "CmpOp") -> cmpop:
        kls = getattr(ast, op, None)
        if kls is None or not issubclass(kls, cmpop):
            raise ValueError(f"Invalid comparison operator: {op}")
        return self.set_position(kls())

    def boolop(self, op: "BoolOp") -> boolop:
        kls = getattr(ast, op, None)
        if kls is None or not issubclass(kls, boolop):
            raise ValueError(f"Invalid boolean operator: {op}")
        return self.set_position(kls())

    def unaryop(self, op: "UnaryOp") -> unaryop:
        kls = getattr(ast, op, None)
        if kls is None or not issubclass(kls, unaryop):
            raise ValueError(f"Invalid unary operator: {op}")
        return self.set_position(kls())

    def set_position(self, node: "T", deep=False) -> "T":
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

    def load(self, node: "str | expr") -> expr:
        """生成一个加载变量的 AST 节点"""
        if isinstance(node, str):
            if node == "None":
                return self.const(None)
            return self.var(node, ctx=self.ctx("Load"))
        return node

    def store(self, node: "str | Name") -> Name:
        """生成一个存储变量的 AST 节点"""
        if isinstance(node, str):
            return self.var(node, ctx=self.ctx("Store"))
        return node

    def var(self, name: str, ctx: "expr_context | None" = None) -> Name:
        """生成一个变量的 AST 节点"""
        return self.set_position(Name(id=name, ctx=ctx or self.ctx("Load")))

    def const_int(self, value: int) -> Constant:
        """生成一个整数常量的 AST 节点"""
        return self.set_position(Constant(value=value))

    def const(self, value: "VmPrimitive" = None) -> expr:
        """生成一个常量的 AST 节点"""
        if is_number(value):
            value = float(value)
        return self.set_position(Constant(value=value))

    def expr(self, value: expr) -> Expr:
        """生成一个表达式语句的 AST 节点"""
        return self.set_position(Expr(value=value))

    def ret(self, value: "expr | str") -> Return:
        """生成一个返回语句的 AST 节点"""
        return self.set_position(Return(value=self.load(value)))

    def pass_stmt(self) -> Pass:
        """生成一个 pass 语句的 AST 节点"""
        return self.set_position(Pass())

    def subscript(
        self, value: "str | expr", slice: expr, ctx: "Ctx" = None
    ) -> Subscript:
        """生成一个下标访问的 AST 节点"""

        # Compatibility: ast.Index was removed in Python 3.9.
        # Use ast.Index for older Pythons, and plain expr (ast.Constant) for 3.9+.
        if sys.version_info < (3, 9):
            from ast import Index

            slice = self.set_position(Index(value=slice))

        return self.set_position(
            Subscript(
                value=self.load(value),
                slice=slice,
                ctx=self.ctx(ctx),
            )
        )

    def assign(self, target: "str | expr | Iterable[expr]", value: expr) -> Assign:
        """生成一个赋值语句的 AST 节点"""
        if isinstance(target, str):
            target = self.store(target)
        return self.set_position(
            Assign(
                targets=[target] if isinstance(target, expr) else list(target),
                value=value,
            )
        )

    def call(
        self, func: "expr | str", args: "Iterable[expr | str] | None" = None
    ) -> Call:
        """生成一个函数调用的 AST 节点"""
        if args is None:
            args = []
        return self.set_position(
            Call(
                func=self.load(func),
                args=[self.load(arg) for arg in args],
                keywords=[],
            )
        )

    def starred(self, value: "expr | str", ctx: "Ctx" = None) -> Starred:
        """生成一个星号表达式的 AST 节点"""
        return self.set_position(Starred(value=self.load(value), ctx=self.ctx(ctx)))

    def tuple(self, elements: "Iterable[expr | str]", ctx: "Ctx" = None) -> Tuple:
        """生成一个元组的 AST 节点"""
        elts = [
            self.var(e, ctx=self.ctx(ctx)) if isinstance(e, str) else e
            for e in elements
        ]
        return self.set_position(Tuple(elts=elts, ctx=self.ctx(ctx)))

    def dict(
        self,
        items: "Iterable[tuple[expr | str, expr | str]]",
        ctx: "Ctx" = None,
    ) -> Dict:
        """生成一个字典的 AST 节点"""
        keys = []
        values = []
        for k, v in items:
            if isinstance(k, str):
                k = self.const(k)
            if isinstance(v, str):
                v = self.var(v, ctx=self.ctx(ctx))
            keys.append(k)
            values.append(v)
        return self.set_position(Dict(keys=keys, values=values))

    def list(self, elements: "Iterable[expr | str]", ctx: "Ctx" = None) -> List:
        """生成一个列表的 AST 节点"""
        elts = [
            self.var(e, ctx=self.ctx(ctx)) if isinstance(e, str) else e
            for e in elements
        ]
        return self.set_position(List(elts=elts, ctx=self.ctx(ctx)))

    def assign_call(
        self,
        target: "str | expr | Iterable[expr]",
        func: "expr | str",
        args: "Iterable[expr | str] | None" = None,
    ) -> Assign:
        """生成一个赋值调用的 AST 节点"""
        call_node = self.call(func, args)
        return self.assign(target, call_node)

    def aug_assign(self, target: "str | Name", op: operator, value: expr) -> AugAssign:
        """生成一个增强赋值的 AST 节点"""
        return self.set_position(
            AugAssign(
                target=self.store(target),
                op=op,
                value=value,
            )
        )

    def func_def(
        self,
        name: str,
        args: "arguments | None" = None,
        body: "Iterable[stmt] | None" = None,
        decorator_list: "Iterable[expr | str] | None" = None,
    ) -> FunctionDef:
        """生成一个函数定义的 AST 节点"""
        return self.set_position(
            FunctionDef(
                name=name,
                args=args or self.args(),
                body=list(body or []),
                decorator_list=[self.load(d) for d in (decorator_list or [])],
            )
        )

    def class_def(
        self,
        name: str,
        body: "Iterable[stmt] | None" = None,
        decorator_list: "Iterable[expr | str] | None" = None,
    ) -> ast.ClassDef:
        """生成一个类定义的 AST 节点"""
        return self.set_position(
            ast.ClassDef(
                name=name,
                bases=[],
                keywords=[],
                body=list(body or []),
                decorator_list=[self.load(d) for d in (decorator_list or [])],
            )
        )

    def compare(
        self, left: "str | expr", op: "cmpop | CmpOp", comparator: expr
    ) -> Compare:
        """生成一个比较表达式的 AST 节点"""
        if isinstance(op, str):
            op = self.cmpop(op)
        return self.set_position(
            Compare(
                left=self.load(left),
                ops=[op],
                comparators=[comparator],
            )
        )

    def binary(
        self, left: "str | expr", op: "operator | Op", right: "str | expr"
    ) -> BinOp:
        """生成一个二元表达式的 AST 节点"""
        if isinstance(op, str):
            op = self.op(op)
        return self.set_position(
            BinOp(
                left=self.load(left),
                op=op,
                right=self.load(right),
            )
        )

    def args(
        self,
        args: "Iterable[str] | None" = None,
        vararg: "str | None" = "vargs",
        kwarg: "str | None" = "kwargs",
    ) -> arguments:
        """生成一个参数列表的 AST 节点"""
        if args is None:
            args = []
        return self.set_position(
            arguments(
                posonlyargs=[],
                args=[self.arg(a) for a in args],
                defaults=[],
                vararg=self.arg(vararg) if vararg else None,
                kwarg=self.arg(kwarg) if kwarg else None,
                kw_defaults=[],
                kwonlyargs=[],
            )
        )

    def arg(self, name: str) -> arg:
        """生成一个参数的 AST 节点"""
        return self.set_position(arg(arg=name, annotation=None))

    def if_expr(self, test: expr) -> If:
        """生成一个 if 表达式的 AST 节点"""
        return self.set_position(If(test=test, body=[], orelse=[]))

    def while_expr(self, test: expr) -> While:
        """生成一个 while 表达式的 AST 节点"""
        return self.set_position(While(test=test, body=[], orelse=[]))

    def for_expr(self, target: "str | Name", iter: "str | expr") -> For:
        """生成一个 for 表达式的 AST 节点"""
        return self.set_position(
            For(
                target=self.store(target),
                iter=self.load(iter),
                body=[],
                orelse=[],
            )
        )

    def uninitialized(self) -> expr:
        """生成一个未初始化的虚拟机值的 AST 节点"""
        return self.load("Uninitialized")

    def vm_hint(self, hint: "str | None" = None) -> "Expr | Pass":
        """生成一个虚拟机提示的 AST 节点"""
        if not hint and self.lineno > 0:
            source = (
                self.source_lines[self.lineno - 1]
                if self.source_lines and self.lineno - 1 < len(self.source_lines)
                else ""
            )
            if source:
                hint = f"{self.lineno}: {source.strip()}"
            else:
                hint = f"{self.lineno}"
        if not hint:
            return self.pass_stmt()
        return self.expr(self.const(hint))

    def vm_element(
        self, args: "str | Iterable[str | expr]", helper_name="Element", spread=False
    ) -> "Call | Starred":
        """生成一个虚拟机元素的 AST 节点"""
        if isinstance(args, str):
            args = [args]
        call = self.call(helper_name, args)
        if spread:
            return self.starred(call)
        else:
            return call

    def vm_if(self, test: "expr | str", negate: bool) -> If:
        """生成一个虚拟机条件表达式的 AST 节点"""

        return self.if_expr(
            self.compare(self.call("ToBoolean", [test]), "NotEq", self.const(negate)),
        )

    def vm_loop_control(self, result: str) -> stmt:
        """生成一个虚拟机循环控制的 AST 节点"""

        is_continue = self.compare(result, "Is", self.load("LoopContinue"))
        chk_continue = self.if_expr(is_continue)
        chk_continue.body.append(self.set_position(ast.Continue()))

        is_break = self.compare(result, "Is", self.load("LoopBreak"))
        chk_break = self.if_expr(is_break)
        chk_break.body.append(self.set_position(ast.Break()))

        chk_continue.orelse.append(chk_break)
        chk_break.orelse.append(self.ret(result))

        return chk_continue

    def vm_range_loop(
        self, index: str, start: str, end: str, exclusive=False
    ) -> "tuple[Sequence[stmt], While]":
        """生成一个虚拟机范围循环的 AST 节点"""
        start_name = f"start_{self.lineno}"
        end_name = f"end_{self.lineno}"
        prepare = []
        hint = self.vm_hint()
        if not isinstance(hint, Pass):
            prepare.append(hint)
        prepare.append(self.assign_call(start_name, "ToNumber", [self.load(start)]))
        prepare.append(self.assign_call(end_name, "ToNumber", [self.load(end)]))
        prepare.append(self.assign(index, self.load(start_name)))
        return prepare, self.while_expr(
            self.compare(
                self.load(index),
                "LtE" if not exclusive else "Lt",
                self.load(end_name),
            )
        )
