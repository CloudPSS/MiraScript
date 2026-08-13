import { DiagnosticCode } from '@mirascript/mirascript/subtle';
import type { editor, languages, CancellationToken, IRange, Position } from '../../../monaco-api.js';
import { Provider } from '../base.js';
import { strictContainsPosition, wordAt } from '../../monaco-utils.js';
import { codeblock, valueDoc } from '../../utils.js';
import { COMMON_GLOBAL_SUGGESTIONS, kwSuggestion } from './common-global-suggestions.js';
import type { CustomCompletionItem } from './interface.js';
import { toCompletionItemRanges } from './utils.js';
import { completeFields } from './complete-fields.js';
import { completeGlobal } from './complete-global.js';
import { completeLocal } from './complete-local.js';

/**
 * 自动完成
 */
export class CompletionItemProvider extends Provider implements languages.CompletionItemProvider {
    readonly triggerCharacters: string[] = ['.', ':'];

    /** @inheritdoc */
    async provideCompletionItems(
        model: editor.ITextModel,
        position: Position,
        context: languages.CompletionContext,
        token: CancellationToken,
    ): Promise<languages.CompletionList | undefined> {
        const compiled = await this.getCompileResult(model);
        if (!compiled) return undefined;

        if (context.triggerCharacter === '.') {
            const prevWord = model.getWordAtPosition({
                lineNumber: position.lineNumber,
                column: position.column - 1,
            });
            if (prevWord?.word === 'global') {
                const globals = await completeGlobal(
                    model,
                    undefined,
                    [],
                    toCompletionItemRanges(position, {
                        startLineNumber: position.lineNumber,
                        startColumn: position.column,
                        endLineNumber: position.lineNumber,
                        endColumn: position.column,
                    }),
                    true,
                );
                return { suggestions: globals };
            }
        }

        const word = wordAt(model, position);
        const prev = model.getValueInRange({
            startLineNumber: position.lineNumber,
            startColumn: (word?.range.startColumn ?? position.column) - 2,
            endLineNumber: position.lineNumber,
            endColumn: word?.range.startColumn ?? position.column,
        });

        if (prev !== '::' && context.triggerCharacter === ':') {
            return undefined; // 不是 :: 触发的
        }

        // suggest variables
        let char: string | undefined;
        let range: IRange;
        const def = compiled.variableAccessAt(model, position);
        if (def) {
            if (def.ref == null) {
                // 输入位置是变量定义
                const suggestions: languages.CompletionItem[] = [];
                if (
                    word &&
                    compiled.tags.some(
                        (t) => strictContainsPosition(t.range, position) && t.code === DiagnosticCode.MatchExpression,
                    )
                ) {
                    suggestions.push(
                        kwSuggestion('case', toCompletionItemRanges(position, word.range)),
                        kwSuggestion('if', toCompletionItemRanges(position, word.range)),
                    );
                }
                return { suggestions };
            }
            const d = def.def;
            range = d.references[def.ref]!.range;
            char = model.getValueInRange({
                startLineNumber: range.startLineNumber,
                startColumn: range.startColumn,
                endLineNumber: range.startLineNumber,
                endColumn: range.startColumn + 1,
            });
        } else if (word) {
            range = word.range;
            char = word.word[0];
        } else {
            range = {
                startLineNumber: position.lineNumber,
                startColumn: position.column,
                endLineNumber: position.lineNumber,
                endColumn: position.column,
            };
        }
        char = char?.toLowerCase();

        const completionRange = toCompletionItemRanges(position, range);

        if (/[^.]\.$/u.test(prev)) {
            const suggestions = await completeFields(model, position, char, completionRange);
            return { suggestions };
        }

        const suggestions = COMMON_GLOBAL_SUGGESTIONS(completionRange, prev === '::');
        const locals = await completeLocal(model, position, char, completionRange);
        const globals = await completeGlobal(model, char, locals, completionRange, false);
        suggestions.push(...locals, ...globals);

        return { suggestions };
    }

    /** @inheritdoc */
    resolveCompletionItem(
        item: languages.CompletionItem,
        token: CancellationToken,
    ): languages.CompletionItem | undefined {
        if (typeof item.label == 'string') {
            // not a dynamic completion item
            return item;
        }
        const { vmValue, isField, vmParent } = item as CustomCompletionItem;
        const { label } = item.label;
        if (vmValue !== undefined || vmParent) {
            if (item.documentation) return item;
            const last = label.split('.').pop()!;
            const def = valueDoc(last, vmValue, isField ? 'field' : 'hint', vmParent ?? null);
            item.documentation = {
                value: `${codeblock('\0' + def.script)}\n${def.doc.join('\n')}`,
            };
        }
        return item;
    }
}
