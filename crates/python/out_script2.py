Module(
    body=[
        FunctionDef(
            name='script',
            args=arguments(
                posonlyargs=[],
                args=[
                    arg(arg='context')],
                vararg=arg(arg='args'),
                kwonlyargs=[],
                kw_defaults=[],
                kwarg=arg(arg='kwargs'),
                defaults=[
                    Call(
                        func=Name(id='GlobalFallback', ctx=Load()),
                        args=[],
                        keywords=[])]),
            body=[
                Try(
                    body=[
                        Expr(
                            value=Call(
                                func=Name(id='CpEnter', ctx=Load()),
                                args=[],
                                keywords=[])),
                        Assign(
                            targets=[
                                Tuple(
                                    elts=[
                                        Name(id='_', ctx=Store()),
                                        Name(id='var_1_1', ctx=Store()),
                                        Name(id='var_1_2', ctx=Store()),
                                        Name(id='var_1_3', ctx=Store()),
                                        Name(id='var_1_4', ctx=Store()),
                                        Name(id='var_1_5', ctx=Store()),
                                        Name(id='var_1_6', ctx=Store()),
                                        Name(id='var_1_7', ctx=Store()),
                                        Name(id='var_1_8', ctx=Store())],
                                    ctx=Store())],
                            value=Tuple(
                                elts=[
                                    Name(id='Uninitialized', ctx=Load()),
                                    Name(id='Uninitialized', ctx=Load()),
                                    Name(id='Uninitialized', ctx=Load()),
                                    Name(id='Uninitialized', ctx=Load()),
                                    Name(id='Uninitialized', ctx=Load()),
                                    Name(id='Uninitialized', ctx=Load()),
                                    Name(id='Uninitialized', ctx=Load()),
                                    Name(id='Uninitialized', ctx=Load()),
                                    Name(id='Uninitialized', ctx=Load())],
                                ctx=Load())),
                        Assign(
                            targets=[
                                Name(id='var_1_5', ctx=Store())],
                            value=Call(
                                func=Name(id='GetGlobal_', ctx=Load()),
                                args=[
                                    Name(id='context', ctx=Load()),
                                    Constant(value='matrix')],
                                keywords=[])),
                        Assign(
                            targets=[
                                Name(id='var_1_5', ctx=Store())],
                            value=Call(
                                func=Name(id='Get_', ctx=Load()),
                                args=[
                                    Name(id='var_1_5', ctx=Load()),
                                    Constant(value='identity')],
                                keywords=[])),
                        If(
                            test=Compare(
                                left=Name(id='var_1_5', ctx=Load()),
                                ops=[
                                    IsNot()],
                                comparators=[
                                    Constant(value=None)]),
                            body=[
                                Assign(
                                    targets=[
                                        Name(id='var_1_6', ctx=Store())],
                                    value=Constant(value=1001.0)),
                                Assign(
                                    targets=[
                                        Name(id='var_1_4', ctx=Store())],
                                    value=Call(
                                        func=Name(id='Call_', ctx=Load()),
                                        args=[
                                            Name(id='var_1_5', ctx=Load()),
                                            Starred(
                                                value=Tuple(
                                                    elts=[
                                                        Name(id='var_1_6', ctx=Load())],
                                                    ctx=Load()),
                                                ctx=Load())],
                                        keywords=[]))],
                            orelse=[
                                Assign(
                                    targets=[
                                        Name(id='var_1_4', ctx=Store())],
                                    value=Constant(value=None))]),
                        FunctionDef(
                            name='var_1_7',
                            args=arguments(
                                posonlyargs=[],
                                args=[
                                    arg(arg='var_2_1')],
                                vararg=arg(arg='args'),
                                kwonlyargs=[],
                                kw_defaults=[],
                                kwarg=arg(arg='kwargs'),
                                defaults=[
                                    Constant(value=None)]),
                            body=[
                                Try(
                                    body=[
                                        Expr(
                                            value=Call(
                                                func=Name(id='CpEnter', ctx=Load()),
                                                args=[],
                                                keywords=[])),
                                        Assign(
                                            targets=[
                                                Tuple(
                                                    elts=[
                                                        Name(id='_', ctx=Store()),
                                                        Name(id='var_2_2', ctx=Store())],
                                                    ctx=Store())],
                                            value=Tuple(
                                                elts=[
                                                    Name(id='Uninitialized', ctx=Load()),
                                                    Name(id='Uninitialized', ctx=Load())],
                                                ctx=Load())),
                                        Assign(
                                            targets=[
                                                Name(id='var_2_2', ctx=Store())],
                                            value=Call(
                                                func=Name(id='Call_', ctx=Load()),
                                                args=[
                                                    Subscript(
                                                        value=Name(id='context', ctx=Load()),
                                                        slice=Constant(value='sum'),
                                                        ctx=Load()),
                                                    Starred(
                                                        value=Tuple(
                                                            elts=[
                                                                Name(id='var_2_1', ctx=Load())],
                                                            ctx=Load()),
                                                        ctx=Load())],
                                                keywords=[])),
                                        Return(
                                            value=Name(id='var_2_2', ctx=Load()))],
                                    handlers=[],
                                    orelse=[],
                                    finalbody=[
                                        Expr(
                                            value=Call(
                                                func=Name(id='CpExit', ctx=Load()),
                                                args=[],
                                                keywords=[]))])],
                            decorator_list=[]),
                        Assign(
                            targets=[
                                Name(id='var_1_3', ctx=Store())],
                            value=Call(
                                func=Name(id='Call_', ctx=Load()),
                                args=[
                                    Subscript(
                                        value=Name(id='context', ctx=Load()),
                                        slice=Constant(value='map'),
                                        ctx=Load()),
                                    Starred(
                                        value=Tuple(
                                            elts=[
                                                Name(id='var_1_4', ctx=Load()),
                                                Name(id='var_1_7', ctx=Load())],
                                            ctx=Load()),
                                        ctx=Load())],
                                keywords=[])),
                        Assign(
                            targets=[
                                Name(id='var_1_2', ctx=Store())],
                            value=Call(
                                func=Name(id='Call_', ctx=Load()),
                                args=[
                                    Subscript(
                                        value=Name(id='context', ctx=Load()),
                                        slice=Constant(value='sum'),
                                        ctx=Load()),
                                    Starred(
                                        value=Tuple(
                                            elts=[
                                                Name(id='var_1_3', ctx=Load())],
                                            ctx=Load()),
                                        ctx=Load())],
                                keywords=[])),
                        Assign(
                            targets=[
                                Name(id='var_1_8', ctx=Store())],
                            value=Constant(value=1001.0)),
                        Assign(
                            targets=[
                                Name(id='_', ctx=Store())],
                            value=Call(
                                func=Name(id='Call_', ctx=Load()),
                                args=[
                                    Subscript(
                                        value=Name(id='context', ctx=Load()),
                                        slice=Constant(value='t_eq'),
                                        ctx=Load()),
                                    Starred(
                                        value=Tuple(
                                            elts=[
                                                Name(id='var_1_2', ctx=Load()),
                                                Name(id='var_1_8', ctx=Load())],
                                            ctx=Load()),
                                        ctx=Load())],
                                keywords=[])),
                        Assign(
                            targets=[
                                Name(id='var_1_1', ctx=Store())],
                            value=Constant(value=None)),
                        Return(
                            value=Name(id='var_1_1', ctx=Load()))],
                    handlers=[],
                    orelse=[],
                    finalbody=[
                        Expr(
                            value=Call(
                                func=Name(id='CpExit', ctx=Load()),
                                args=[],
                                keywords=[]))])],
            decorator_list=[])],
    type_ignores=[])