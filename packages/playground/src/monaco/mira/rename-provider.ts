import { type CancellationToken, type editor, languages, type Position, Range } from '@private/monaco-editor';
import { Provider } from './worker-helper';
import { DiagnosticCode } from 'mira-wasm';

/** @inheritdoc */
class RenameProvider extends Provider implements languages.RenameProvider {
    /** @inheritdoc */
    async provideRenameEdits(
        model: editor.ITextModel,
        position: Position,
        newName: string,
        token: CancellationToken,
    ): Promise<undefined | (languages.WorkspaceEdit & languages.Rejection)> {
        const compiled = await Provider.getCompileResult(model);
        if (!compiled) return undefined;
        const { omitNameFields } = compiled.groupedTags(model);
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

        return { edits };
    }

    /** @inheritdoc */
    async resolveRenameLocation(
        model: editor.ITextModel,
        position: Position,
        token: CancellationToken,
    ): Promise<undefined | (languages.RenameLocation & languages.Rejection)> {
        const compiled = await Provider.getCompileResult(model);
        if (!compiled) return undefined;
        const d = compiled.definition(model, position);
        if (!d) {
            return {
                range: Range.fromPositions(position),
                text: '',
                rejectReason: 'Cannot rename this element',
            };
        }
        const { def, ref } = d;
        if ('name' in def) {
            return {
                range: def.references[ref!]?.range ?? Range.fromPositions(position),
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

languages.registerRenameProvider('mirascript', new RenameProvider());
