import { DiagnosticCode } from '@mirascript/constants';
import { type editor, type languages, Range, type CancellationToken } from '../../monaco-api.js';
import { getDiagnosticCode } from '../diagnostics.js';
import { Provider, type MonacoContext } from './base.js';
import type { CompileResult } from '../compile-result.js';

/** Create code action based on diagnostic marker */
function createCodeAction(
    result: CompileResult,
    context: MonacoContext,
    model: editor.ITextModel,
    marker: editor.IMarkerData,
): languages.CodeAction | undefined {
    const code = getDiagnosticCode(marker);
    if (!code) return undefined;

    const range = Range.lift(marker);
    const action: languages.CodeAction = {
        title: '',
        diagnostics: [marker],
    };
    const edits = (
        ...textEdits: Array<languages.IWorkspaceTextEdit['textEdit'] | undefined>
    ): languages.WorkspaceEdit => {
        const edits: languages.IWorkspaceTextEdit[] = [];
        for (const textEdit of textEdits) {
            if (textEdit == null) continue;
            edits.push({
                resource: model.uri,
                versionId: result.version,
                textEdit,
            });
        }
        return { edits };
    };

    if (code === DiagnosticCode.PreferUppercaseConstant) {
        const current = model.getValueInRange(range);
        const uppercase = current.toUpperCase();
        if (current === uppercase || !Object.is(context.getOrUndefined(current), context.getOrUndefined(uppercase))) {
            return undefined;
        }
        action.title = '转换为大写常量';
        action.kind = 'quickfix';
        action.edit = edits({
            range,
            text: uppercase,
        });
    } else if (code === DiagnosticCode.PreferParenthesesForRecordLiteral) {
        const current = model.getValueInRange(range);
        let fixed = current;
        if (current.startsWith('{ ')) {
            fixed = `(${fixed.slice(2)}`;
        } else if (current.startsWith('{')) {
            fixed = `(${fixed.slice(1)}`;
        }
        if (current.endsWith(' }')) {
            fixed = `${fixed.slice(0, -2)})`;
        } else if (current.endsWith('}')) {
            fixed = `${fixed.slice(0, -1)})`;
        }
        if (fixed === current) {
            return undefined;
        }
        const prefix = model.getValueInRange({
            startLineNumber: range.startLineNumber,
            startColumn: range.startColumn - 1,
            endLineNumber: range.startLineNumber,
            endColumn: range.startColumn,
        });
        if (prefix === '$') {
            // In string "${...}", we should add extra parentheses
            // to "$((...))" to represent a record literal
            fixed = `(${fixed})`;
        }
        action.title = `转换为使用圆括号的记录字面量`;
        action.kind = 'quickfix';
        action.edit = edits({
            range,
            text: fixed,
        });
    } else if (
        code === DiagnosticCode.PreferLogicalOperatorAnd ||
        code === DiagnosticCode.PreferLogicalOperatorOr ||
        code === DiagnosticCode.PreferLogicalOperatorNot
    ) {
        let range0 = range;
        const current = model.getValueInRange(range);
        let fixed: string | undefined;
        if (current === 'and') {
            fixed = '&&';
        } else if (current === 'or') {
            fixed = '||';
        } else if (current === 'not') {
            const range1 = range.setEndPosition(range.endLineNumber, range.endColumn + 1);
            const current1 = model.getValueInRange(range1);
            if (current1.trim() === 'not') {
                range0 = range1;
            }
            fixed = '!';
        }
        if (!fixed) {
            return undefined;
        }
        action.title = `转换为 '${fixed}' 运算符`;
        action.kind = 'quickfix';
        action.edit = edits({
            range: range0,
            text: fixed,
        });
    } else if (code === DiagnosticCode.UnnecessaryParentheses) {
        const current = model.getValueInRange(range);
        let fixed = current;
        if (current.startsWith('(')) {
            fixed = fixed.slice(1);
        }
        if (current.endsWith(')')) {
            fixed = fixed.slice(0, -1);
        }
        if (fixed === current) {
            return undefined;
        }
        action.title = '移除不必要的括号';
        action.kind = 'quickfix';
        action.edit = edits({
            range,
            text: fixed,
        });
    } else if (code === DiagnosticCode.PreferIfExpression) {
        const tag = result.tags.find(
            (tag) => tag.code === DiagnosticCode.IfExpression && Range.equalsRange(tag.range, range),
        );
        if (!tag) return undefined;
        const questionMark = tag.references.find((ref) => ref.code === DiagnosticCode.KeywordIf);
        const colon = tag.references.find((ref) => ref.code === DiagnosticCode.KeywordElse);
        if (!questionMark || !colon) return undefined;
        const condRange = Range.fromPositions(
            range.getStartPosition(),
            model.getPositionAt(model.getOffsetAt(Range.getStartPosition(questionMark.range))),
        );
        const thenRange = Range.fromPositions(
            model.getPositionAt(model.getOffsetAt(Range.getEndPosition(questionMark.range))),
            model.getPositionAt(model.getOffsetAt(Range.getStartPosition(colon.range))),
        );
        const elseRange = Range.fromPositions(
            model.getPositionAt(model.getOffsetAt(Range.getEndPosition(colon.range))),
            range.getEndPosition(),
        );
        const cond = model.getValueInRange(condRange).trim();
        const thenExpr = model.getValueInRange(thenRange).trim();
        const elseExpr = model.getValueInRange(elseRange).trim();
        const wrapIfNeeded = (expr: string, elseBranch: boolean): string => {
            // If expression is allowed in else branch without parentheses
            if (elseBranch && expr.startsWith('if ') && expr.endsWith('}')) {
                return expr;
            }
            // Block expression, but not JSON-like record literal
            if (expr.startsWith('{') && expr.endsWith('}') && !expr.includes(':')) {
                return expr;
            }
            if (expr.includes('\n')) {
                return `{\n  ${expr}\n}`;
            }
            return `{ ${expr} }`;
        };
        const fixed = `if ${cond} ${wrapIfNeeded(thenExpr, false)} else ${wrapIfNeeded(elseExpr, true)}`;
        action.title = '转换为 if 表达式';
        action.kind = 'quickfix';
        action.edit = edits({
            range,
            text: fixed,
        });
    }

    if (action.title) return action;
    return undefined;
}

/**
 * 代码操作
 */
export class CodeActionProvider extends Provider implements languages.CodeActionProvider {
    /** @inheritdoc */
    async provideCodeActions(
        model: editor.ITextModel,
        range: Range,
        { markers, only, trigger }: languages.CodeActionContext,
        token: CancellationToken,
    ): Promise<languages.CodeActionList | undefined> {
        const result = await this.getCompileResult(model);
        if (!result) return undefined;
        const context = await this.getContext(model);
        const actions = [];
        for (const marker of markers) {
            const action = createCodeAction(result, context, model, marker);
            if (action == null || (only && action.kind ? action.kind !== only : false)) {
                continue;
            }
            actions.push(action);
        }
        return {
            actions,
            dispose: () => void 0,
        };
    }

    /** @inheritdoc */
    resolveCodeAction?(
        codeAction: languages.CodeAction,
        token: CancellationToken,
    ): languages.ProviderResult<languages.CodeAction>;
}
