import type { CancellationToken, editor, IRange, languages, Position } from '@private/monaco-editor';
import { DiagnosticCode } from '@mirascript/wasm';
import { Provider } from './base.js';
import type { CompileResult } from '../compile-result.js';

/** @inheritdoc */
export class RenameProvider extends Provider implements languages.RenameProvider {
    /** 重命名推断字段 */
    private provideRenameEditsOmitNameFields(
        model: editor.ITextModel,
        compiled: CompileResult,
        edits: languages.IWorkspaceTextEdit[],
        ref: { range: IRange },
        oldName: string,
    ): void {
        const { omitNameFields } = compiled.groupedTags(model);
        const { Range } = this.monaco;
        for (const tag of omitNameFields) {
            if (Range.equalsRange(tag.references[0]?.range, ref.range)) {
                const current = model.getValueInRange(tag.range);
                const paddingLeft = /[^\s(]/u.test(
                    model.getValueInRange({
                        startLineNumber: tag.range.startLineNumber,
                        startColumn: tag.range.startColumn - 1,
                        endLineNumber: tag.range.startLineNumber,
                        endColumn: tag.range.startColumn,
                    }),
                );
                const paddingRight = !/\s/u.test(
                    model.getValueInRange({
                        startLineNumber: tag.range.endLineNumber,
                        startColumn: tag.range.endColumn,
                        endLineNumber: tag.range.endLineNumber,
                        endColumn: tag.range.endColumn + 1,
                    }),
                );
                edits.push({
                    resource: model.uri,
                    versionId: compiled.version,
                    textEdit: {
                        range: tag.range,
                        text: `${paddingLeft ? ' ' : ''}${oldName}${current}${paddingRight ? ' ' : ''}`,
                    },
                });
            }
        }
    }
    /** @inheritdoc */
    async provideRenameEdits(
        model: editor.ITextModel,
        position: Position,
        newName: string,
        token: CancellationToken,
    ): Promise<undefined | (languages.WorkspaceEdit & languages.Rejection)> {
        const compiled = await this.getCompileResult(model);
        if (!compiled) return undefined;
        const d = compiled.definition(model, position);
        if (!d) return undefined;
        const edits: languages.IWorkspaceTextEdit[] = [];
        const { references } = d.def;
        let oldName;
        if ('definition' in d.def) {
            edits.push({
                resource: model.uri,
                versionId: compiled.version,
                textEdit: {
                    range: d.def.definition.range,
                    text: d.def.definition.code === DiagnosticCode.ParameterIt ? `(${newName})` : newName,
                },
            });
            oldName = model.getValueInRange(d.def.definition.range);
            this.provideRenameEditsOmitNameFields(model, compiled, edits, d.def.definition, oldName);
        } else {
            oldName = d.def.name;
        }
        for (const ref of references) {
            edits.push({
                resource: model.uri,
                versionId: compiled.version,
                textEdit: {
                    range: ref.range,
                    text: newName,
                },
            });
            this.provideRenameEditsOmitNameFields(model, compiled, edits, ref, oldName);
        }

        return { edits };
    }

    /** @inheritdoc */
    async resolveRenameLocation(
        model: editor.ITextModel,
        position: Position,
        token: CancellationToken,
    ): Promise<undefined | (languages.RenameLocation & languages.Rejection)> {
        const compiled = await this.getCompileResult(model);
        if (!compiled) return undefined;
        const d = compiled.definition(model, position);
        if (!d) {
            return {
                range: this.monaco.Range.fromPositions(position),
                text: '',
                rejectReason: 'Cannot rename this element',
            };
        }
        const { def, ref } = d;
        if ('name' in def) {
            return {
                range: def.references[ref!]?.range ?? this.monaco.Range.fromPositions(position),
                text: def.name,
                rejectReason: 'Cannot rename global variables',
            };
        }
        const tag = ref == null ? def.definition : def.references[ref]!;
        return {
            range: tag.range,
            text: model.getValueInRange(tag.range) ?? '',
        };
    }
}
