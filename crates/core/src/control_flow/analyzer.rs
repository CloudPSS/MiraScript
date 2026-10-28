use crate::{
    DiagnosticCode, Keyword, Operator, SourceRange,
    diagnostic::DiagnosticsCollector,
    parser::{
        ArrayElementBase, AstWalker, Callable, ElseBlock, Expression, Iterable, MatchCase, Pattern,
        RecordElementBase, Statement,
    },
};

use super::Flow;

pub(super) struct Analyzer<'a, 's, 'c> {
    diagnostics: &'a mut DiagnosticsCollector<'s, 'c>,
}

impl<'a, 's, 'c> Analyzer<'a, 's, 'c> {
    pub(super) fn new(diagnostics: &'a mut DiagnosticsCollector<'s, 'c>) -> Self {
        Self { diagnostics }
    }
}

impl Analyzer<'_, '_, '_> {
    fn unreachable(&mut self, range: SourceRange) {
        if !range.is_empty() {
            self.diagnostics
                .push(DiagnosticCode::UnreachableCode, range);
        }
    }

    fn flush(&mut self, range: &mut Option<SourceRange>) {
        if let Some(range) = range.take() {
            self.unreachable(range);
        }
    }

    pub(super) fn block(
        &mut self,
        stmts: &[Statement<'_>],
        tail: Option<&Expression<'_>>,
        in_loop: bool,
    ) -> Flow {
        let mut flow = Flow::NORMAL;
        let mut dead: Option<SourceRange> = None;
        for stmt in stmts {
            // Function declarations are initialized at block entry, even after a jump.
            if let Statement::Function(_, _, _, _, body) = stmt {
                self.flush(&mut dead);
                self.expr(body, false);
            } else if flow.has(Flow::NORMAL) {
                flow = flow.then(self.stmt(stmt, in_loop));
            } else if !matches!(stmt, Statement::Empty(_)) {
                let range = stmt.range();
                dead.get_or_insert(range.clone()).end = range.end;
            }
        }
        if let Some(tail) = tail {
            if flow.has(Flow::NORMAL) {
                flow = flow.then(self.expr(tail, in_loop));
            } else {
                let range = tail.range();
                dead.get_or_insert(range.clone()).end = range.end;
            }
        }
        self.flush(&mut dead);
        flow
    }

    fn stmt(&mut self, stmt: &Statement<'_>, in_loop: bool) -> Flow {
        use Statement::*;
        match stmt {
            Expression(expr, _)
            | BlockExpression(expr)
            | Module(_, _, _, expr)
            | Bind(_, _, _, _, expr, _)
            | Rebind(_, _, expr, _)
            | Const(_, _, _, _, expr, _) => self.expr(expr, in_loop),
            Assign(left, op, right, _) => {
                let left = self.expr(left, in_loop);
                let right_flow = self.next(left, right, in_loop);
                let optional = **op == Operator::LogicalAndAssign
                    || **op == Operator::LogicalOrAssign
                    || **op == Operator::NullCoalescingAssign;
                left.then(if optional {
                    right_flow.union(Flow::NORMAL)
                } else {
                    right_flow
                })
            }
            Return(_, value, _) | Break(_, value, _) => {
                let flow = value
                    .as_deref()
                    .map_or(Flow::NORMAL, |e| self.expr(e, in_loop));
                let exit = if matches!(stmt, Return(..)) {
                    Flow::RETURN
                } else if in_loop {
                    Flow::BREAK
                } else {
                    Flow::NORMAL
                };
                flow.then(exit)
            }
            Continue(..) if in_loop => Flow::CONTINUE,
            Function(..) | Continue(..) | Empty(_) | Unknown { .. } => Flow::NORMAL,
        }
    }

    /// Evaluate a mandatory next operand, or mark it as dead without descending into it.
    fn next(&mut self, flow: Flow, expr: &Expression<'_>, in_loop: bool) -> Flow {
        if flow.has(Flow::NORMAL) {
            self.expr(expr, in_loop)
        } else {
            self.unreachable(expr.range());
            Flow::NEVER
        }
    }

    fn sequence<'e, 's: 'e>(
        &mut self,
        exprs: impl IntoIterator<Item = &'e Expression<'s>>,
        in_loop: bool,
    ) -> Flow {
        let mut flow = Flow::NORMAL;
        for expr in exprs {
            let next = self.next(flow, expr, in_loop);
            flow = flow.then(next);
        }
        flow
    }

    fn iterable(&mut self, iterable: &Iterable<'_>, in_loop: bool) -> Flow {
        match iterable {
            Iterable::Value(expr) => self.expr(expr, in_loop),
            Iterable::Range(range) => self.sequence([&*range.0, &*range.2], in_loop),
        }
    }

    fn branches(
        &mut self,
        cond: &Expression<'_>,
        yes: &Expression<'_>,
        no: Option<&Expression<'_>>,
        in_loop: bool,
    ) -> Flow {
        let (flow, value) = self.expr_result(cond, in_loop);
        let yes_flow = if !flow.has(Flow::NORMAL) || value == Some(false) {
            self.unreachable(yes.range());
            Flow::NEVER
        } else {
            self.expr(yes, in_loop)
        };
        let no_flow = if !flow.has(Flow::NORMAL) || value == Some(true) {
            if let Some(no) = no {
                self.unreachable(no.range());
            }
            Flow::NEVER
        } else {
            no.map_or(Flow::NORMAL, |no| self.expr(no, in_loop))
        };
        flow.then(yes_flow.union(no_flow))
    }

    fn expr(&mut self, expr: &Expression<'_>, in_loop: bool) -> Flow {
        self.expr_result(expr, in_loop).0
    }

    /// Compute syntactic boolean constants during the same traversal as control flow.
    /// Other expressions deliberately discard their children's constants to stay conservative.
    fn expr_result(&mut self, expr: &Expression<'_>, in_loop: bool) -> (Flow, Option<bool>) {
        use Expression::*;
        let flow = match expr {
            Literal(token) => {
                let value = if **token == Keyword::True {
                    Some(true)
                } else if **token == Keyword::False {
                    Some(false)
                } else {
                    None
                };
                return (Flow::NORMAL, value);
            }
            Variable(_) | Unknown { .. } => Flow::NORMAL,
            Grouping(_, inner, _) => return self.expr_result(inner, in_loop),
            Prefix(op, inner) => {
                let (flow, value) = self.expr_result(inner, in_loop);
                let value = if **op == Operator::Exclamation || **op == Keyword::Not {
                    value.map(|v| !v)
                } else {
                    None
                };
                return (flow, value);
            }
            NonNil(inner, _) | Access(inner, _, _) | Is(inner, _, _) => self.expr(inner, in_loop),
            InterpolatedString(_, exprs) => self.sequence(exprs, in_loop),
            Block(_, stmts, tail, _) => self.block(stmts, tail.as_deref(), in_loop),
            Function(_, _, body) => {
                self.expr(body, false);
                Flow::NORMAL
            }
            Record(_, elements, _) => {
                let mut operands = Vec::new();
                for element in elements {
                    match &**element {
                        RecordElementBase::InterpolateNamed(name, _, value) => {
                            operands.push(&**name);
                            operands.push(&**value);
                        }
                        RecordElementBase::Named(_, _, value)
                        | RecordElementBase::OmitNamed(_, value)
                        | RecordElementBase::Unnamed(value)
                        | RecordElementBase::Spread(_, value) => operands.push(&**value),
                    }
                }
                self.sequence(operands, in_loop)
            }
            Array(_, elements, _) => {
                let mut flow = Flow::NORMAL;
                for element in elements {
                    let next = if !flow.has(Flow::NORMAL) {
                        self.unreachable((**element).range());
                        Flow::NEVER
                    } else {
                        match &**element {
                            ArrayElementBase::Element(value) => self.iterable(value, in_loop),
                            ArrayElementBase::Spread(_, value) => self.expr(value, in_loop),
                        }
                    };
                    flow = flow.then(next);
                }
                flow
            }
            Index(object, _, index, _) => self.sequence([&**object, &**index], in_loop),
            Slice(object, _, start, _, end, _) => self.sequence(
                std::iter::once(&**object)
                    .chain(start.as_deref())
                    .chain(end.as_deref()),
                in_loop,
            ),
            Infix(left, op, right) => {
                let (flow, value) = self.expr_result(left, in_loop);
                let and = **op == Operator::LogicalAnd || **op == Keyword::And;
                let or = **op == Operator::LogicalOr || **op == Keyword::Or;
                if !flow.has(Flow::NORMAL)
                    || (and && value == Some(false))
                    || (or && value == Some(true))
                {
                    self.unreachable(right.range());
                    return (flow, if and || or { value } else { None });
                }
                let (right, right_value) = self.expr_result(right, in_loop);
                let optional = ((and || or) && value.is_none()) || **op == Operator::NullCoalescing;
                let flow = flow.then(if optional {
                    right.union(Flow::NORMAL)
                } else {
                    right
                });
                let value = if (and && value == Some(true)) || (or && value == Some(false)) {
                    right_value
                } else {
                    None
                };
                return (flow, value);
            }
            If(_, cond, yes, no) => self.branches(cond, yes, no.as_ref().map(|e| &*e.1), in_loop),
            Cond(cond, _, yes, _, no) => self.branches(cond, yes, Some(no), in_loop),
            Loop(_, body) => {
                let body = self.expr(body, true);
                let exits = body & Flow::RETURN;
                if body.has(Flow::BREAK) {
                    exits.union(Flow::NORMAL)
                } else {
                    exits
                }
            }
            While(_, cond, body, no) => self.while_expr(cond, body, no.as_ref()),
            ForIn(_, _, _, iterable, body, no) => {
                // The emitter evaluates the iterable and loop-else with no break target.
                let input = self.iterable(iterable, false);
                if !input.has(Flow::NORMAL) {
                    self.unreachable(body.range());
                    if let Some(no) = no {
                        self.unreachable(no.1.range());
                    }
                    return (input, None);
                }
                let body = self.expr(body, true);
                let no = no
                    .as_ref()
                    .map_or(Flow::NORMAL, |no| self.expr(&no.1, false));
                let mut result = no.union(body & Flow::RETURN);
                if body.has(Flow::BREAK) {
                    result = result.union(Flow::NORMAL);
                }
                input.then(result)
            }
            Match(_, input, _, cases, _) => self.match_expr(input, cases, in_loop),
            Call(callable, _, args, _) | Extension(_, _, callable, _, args, _) => {
                // Ordinary and extension calls evaluate the callable before their arguments.
                let first = if let Extension(first, ..) = expr {
                    Some(&**first)
                } else {
                    None
                };
                let args = first.into_iter().chain(args.iter().map(|arg| match &**arg {
                    ArrayElementBase::Element(value) | ArrayElementBase::Spread(_, value) => {
                        &**value
                    }
                }));
                match callable {
                    Callable::Type(_) => self.sequence(args, in_loop),
                    Callable::Expression(callable) => self.call(callable, args, in_loop),
                }
            }
            TaggedString(callable, string) => self.call(callable, [&**string], in_loop),
        };
        (flow, None)
    }

    fn call<'e, 's: 'e>(
        &mut self,
        callable: &Expression<'_>,
        args: impl IntoIterator<Item = &'e Expression<'s>>,
        in_loop: bool,
    ) -> Flow {
        let flow = self.expr(callable, in_loop);
        if !flow.has(Flow::NORMAL) {
            for arg in args {
                self.unreachable(arg.range());
            }
            return flow;
        }
        let args = self.sequence(args, in_loop);
        // Complex callees may be nil, in which case the VM skips the arguments.
        let global = matches!(callable, Expression::Access(parent, ..)
            if matches!(&**parent, Expression::Variable(token) if **token == Keyword::Global));
        let optional =
            !global && !matches!(callable, Expression::Variable(_) | Expression::NonNil(..));
        flow.then(if optional {
            args.union(Flow::NORMAL)
        } else {
            args
        })
    }

    fn while_expr(
        &mut self,
        cond: &Expression<'_>,
        body: &Expression<'_>,
        no: Option<&ElseBlock<'_>>,
    ) -> Flow {
        // Unlike for-in's iterable, a while condition executes inside its loop.
        let (cond_flow, value) = self.expr_result(cond, true);
        let body_flow = if cond_flow.has(Flow::NORMAL) && value != Some(false) {
            self.expr(body, true)
        } else {
            self.unreachable(body.range());
            Flow::NEVER
        };
        let no_flow = if cond_flow.has(Flow::NORMAL) && value != Some(true) {
            no.map_or(Flow::NORMAL, |no| self.expr(&no.1, false))
        } else {
            if let Some(no) = no {
                self.unreachable(no.1.range());
            }
            Flow::NEVER
        };
        let mut exits = ((cond_flow | body_flow) & Flow::RETURN).union(no_flow);
        if cond_flow.has(Flow::BREAK) || body_flow.has(Flow::BREAK) {
            exits = exits.union(Flow::NORMAL);
        }
        exits
    }

    fn match_expr(
        &mut self,
        input: &Expression<'_>,
        cases: &[MatchCase<'_>],
        in_loop: bool,
    ) -> Flow {
        let input = self.expr(input, in_loop);
        let mut remaining = input.has(Flow::NORMAL);
        let mut exits = Flow::NEVER;
        for case in cases {
            if !remaining {
                self.unreachable(case.range());
                continue;
            }
            let MatchCase(_, pattern, guard, body) = case;
            let (guard_flow, value) =
                guard.as_ref().map_or((Flow::NORMAL, Some(true)), |(_, e)| {
                    self.expr_result(e, in_loop)
                });
            exits = exits.union(guard_flow.abrupt());
            if guard_flow.has(Flow::NORMAL) && value != Some(false) {
                exits = exits.union(self.expr(body, in_loop));
            } else {
                self.unreachable(body.range());
            }
            if irrefutable(pattern) && (!guard_flow.has(Flow::NORMAL) || value == Some(true)) {
                remaining = false;
            }
        }
        if remaining {
            exits = exits.union(Flow::NORMAL);
        }
        input.then(exits)
    }
}

fn irrefutable(pattern: &Pattern<'_>) -> bool {
    match pattern {
        Pattern::Discard(_) | Pattern::Bind(..) => true,
        Pattern::Grouping(_, inner, _) => irrefutable(inner),
        Pattern::And(left, _, right) => irrefutable(left) && irrefutable(right),
        Pattern::Or(left, _, right) => irrefutable(left) || irrefutable(right),
        _ => false,
    }
}
