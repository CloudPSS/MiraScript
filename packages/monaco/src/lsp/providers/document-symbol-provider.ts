import type { CancellationToken, editor, languages } from '@private/monaco-editor';
import { DiagnosticCode } from '@mirascript/wasm';
import { Provider } from './base.js';
import type { SourceScope } from '../compile-result.js';

/** @inheritdoc */
export class DocumentSymbolProvider extends Provider implements languages.DocumentSymbolProvider {
    /** 构建树 */
    private handleScope(model: editor.ITextModel, scope: SourceScope): languages.DocumentSymbol[] {
        const symbols: languages.DocumentSymbol[] = [];
        const unhandledChildren = new Set(scope.children);
        const { languages, Range } = this.monaco;
        for (const { definition } of scope.locals) {
            const { range } = definition;
            let kind: languages.SymbolKind = languages.SymbolKind.Variable;
            let name: string | undefined;
            let children: languages.DocumentSymbol[] = [];
            let allRange = range;
            // eslint-disable-next-line @typescript-eslint/switch-exhaustiveness-check
            switch (definition.code) {
                case DiagnosticCode.ParameterIt:
                    if (definition.references.length === 0) {
                        // 忽略未使用的 it 参数
                        continue;
                    }
                    name = `it`;
                    break;
                case DiagnosticCode.LocalFunction: {
                    kind = languages.SymbolKind.Function;
                    const funcScope = scope.children.find((s) => Range.compareRangesUsingStarts(s.range, range) > 0);
                    if (funcScope) {
                        allRange = Range.plusRange(range, funcScope.range);
                        children = this.handleScope(model, funcScope);
                        unhandledChildren.delete(funcScope);
                    }
                    break;
                }
            }
            symbols.push({
                name: name ?? model.getValueInRange(definition.range),
                detail: '',
                kind,
                range: allRange,
                tags: [],
                selectionRange: range,
                children: children,
            } satisfies languages.DocumentSymbol);
        }
        // 处理未处理的子作用域
        for (const child of unhandledChildren) {
            symbols.push(...this.handleScope(model, child));
        }
        return symbols;
    }
    /** @inheritdoc */
    async provideDocumentSymbols(
        model: editor.ITextModel,
        token: CancellationToken,
    ): Promise<languages.DocumentSymbol[] | undefined> {
        const compiled = await this.getCompileResult(model);
        if (!compiled) return undefined;
        const root = compiled.scopes(model)[0];
        if (!root) return undefined;
        return this.handleScope(model, root);
    }
}
