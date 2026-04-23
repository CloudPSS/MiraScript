## 将所有 nonlocal 声明提升到函数体顶部,有些情况下nonlocal声明可能在条件语句或循环内
import ast


def deep_nonlocal_fix(node, nonlocal_vars=None):
    """
    将函数体内部所有非嵌套位置的 `nonlocal` 声明收集并提升到函数体顶部。
    对 FunctionDef / AsyncFunctionDef 递归处理，保持嵌套函数的 nonlocal 在其自身作用域内。
    """

    class _Hoister(ast.NodeTransformer):
        def visit_FunctionDef(self, fn: ast.FunctionDef):
            # 先处理内部（从底向上）
            self.generic_visit(fn)

            # 收集当前函数作用域（不进入嵌套函数/类/lambda）的 nonlocal 名称
            names = set()

            def collect_from_nodes(stmts):
                for s in stmts:
                    if isinstance(s, ast.Nonlocal):
                        for n in s.names:
                            names.add(n)
                    # 不进入嵌套作用域
                    elif isinstance(
                        s,
                        (
                            ast.FunctionDef,
                            ast.AsyncFunctionDef,
                            ast.ClassDef,
                            ast.Lambda,
                        ),
                    ):
                        continue
                    else:
                        if s is None:
                            continue
                        for field, value in ast.iter_fields(s):
                            if isinstance(value, list):
                                collect_from_nodes(value)
                            elif isinstance(value, ast.AST):
                                if isinstance(
                                    value,
                                    (
                                        ast.FunctionDef,
                                        ast.AsyncFunctionDef,
                                        ast.ClassDef,
                                        ast.Lambda,
                                    ),
                                ):
                                    continue
                                collect_from_nodes([value])

            collect_from_nodes(fn.body)

            if names:
                # 从当前函数体及其子节点中移除所有 Nonlocal 节点（不进入嵌套函数/类/lambda）
                def remove_from_nodes(stmts):
                    new_stmts = []
                    for s in stmts:
                        if isinstance(s, ast.Nonlocal):
                            # 跳过（已收集）
                            continue
                        elif isinstance(
                            s,
                            (
                                ast.FunctionDef,
                                ast.AsyncFunctionDef,
                                ast.ClassDef,
                                ast.Lambda,
                            ),
                        ):
                            new_stmts.append(s)
                        else:
                            for field, value in list(ast.iter_fields(s)):
                                if isinstance(value, list):
                                    filtered = remove_from_nodes(value)
                                    setattr(s, field, filtered)
                                elif isinstance(value, ast.AST):
                                    if isinstance(
                                        value,
                                        (
                                            ast.FunctionDef,
                                            ast.AsyncFunctionDef,
                                            ast.ClassDef,
                                            ast.Lambda,
                                        ),
                                    ):
                                        pass
                                    else:
                                        replaced = remove_from_nodes([value])
                                        if replaced:
                                            setattr(s, field, replaced[0])
                            new_stmts.append(s)
                    return new_stmts

                fn.body = remove_from_nodes(fn.body)

                # 构造新的 nonlocal 节点并插入到函数体顶部，若首节点为 Try 则插入到 Try.body 顶部
                nl_node = ast.Nonlocal(names=sorted(names))
                if fn.body and isinstance(fn.body[0], ast.Try):
                    try_node: ast.Try = fn.body[0]
                    try_node.body.insert(0, nl_node)
                else:
                    fn.body.insert(0, nl_node)

            return fn

        def visit_AsyncFunctionDef(self, fn: ast.AsyncFunctionDef):
            # 同样处理异步函数
            self.generic_visit(fn)

            names = set()

            def collect_from_nodes(stmts):
                for s in stmts:
                    if isinstance(s, ast.Nonlocal):
                        for n in s.names:
                            names.add(n)
                    elif isinstance(
                        s,
                        (
                            ast.FunctionDef,
                            ast.AsyncFunctionDef,
                            ast.ClassDef,
                            ast.Lambda,
                        ),
                    ):
                        continue
                    else:
                        for field, value in ast.iter_fields(s):
                            if isinstance(value, list):
                                collect_from_nodes(value)
                            elif isinstance(value, ast.AST):
                                if isinstance(
                                    value,
                                    (
                                        ast.FunctionDef,
                                        ast.AsyncFunctionDef,
                                        ast.ClassDef,
                                        ast.Lambda,
                                    ),
                                ):
                                    continue
                                collect_from_nodes([value])

            collect_from_nodes(fn.body)

            if names:

                def remove_from_nodes(stmts):
                    new_stmts = []
                    for s in stmts:
                        if isinstance(s, ast.Nonlocal):
                            continue
                        elif isinstance(
                            s,
                            (
                                ast.FunctionDef,
                                ast.AsyncFunctionDef,
                                ast.ClassDef,
                                ast.Lambda,
                            ),
                        ):
                            new_stmts.append(s)
                        else:
                            for field, value in list(ast.iter_fields(s)):
                                if isinstance(value, list):
                                    filtered = remove_from_nodes(value)
                                    setattr(s, field, filtered)
                                elif isinstance(value, ast.AST):
                                    if isinstance(
                                        value,
                                        (
                                            ast.FunctionDef,
                                            ast.AsyncFunctionDef,
                                            ast.ClassDef,
                                            ast.Lambda,
                                        ),
                                    ):
                                        pass
                                    else:
                                        replaced = remove_from_nodes([value])
                                        if replaced:
                                            setattr(s, field, replaced[0])
                            new_stmts.append(s)
                    return new_stmts

                fn.body = remove_from_nodes(fn.body)
                nl_node = ast.Nonlocal(names=sorted(names), lineno=0)
                if fn.body and isinstance(fn.body[0], ast.Try):
                    fn.body[0].body.insert(0, nl_node)
                else:
                    fn.body.insert(0, nl_node)

            return fn

    hoister = _Hoister()
    return hoister.visit(node)


# ...existing code...
