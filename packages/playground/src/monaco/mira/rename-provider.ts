import { type CancellationToken, type editor, languages, type Position, Range } from '@private/monaco-editor';
import { Provider } from './worker-helper';
import { strictInRange } from './utils';
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
        const d = compiled.definition(model, position);
        if (!d) return undefined;
        const edits: languages.IWorkspaceTextEdit[] = [];
        const { definition, references } = d.def;
        if (definition) {
            edits.push({
                resource: model.uri,
                versionId: compiled.version,
                textEdit: {
                    range: definition.range,
                    text:
                        definition.code === DiagnosticCode.ParameterIt ||
                        definition.code === DiagnosticCode.UnusedParameterIt
                            ? `(${newName})`
                            : newName,
                },
            });
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
        const tags = compiled.tags.filter(
            (t) =>
                t.code !== DiagnosticCode.Scope &&
                t.code !== DiagnosticCode.String &&
                t.code !== DiagnosticCode.Interpolation &&
                strictInRange(t.range, position),
        );
        if (!tags.length) {
            return {
                range: Range.fromPositions(position),
                text: '',
                rejectReason: 'Cannot rename this element',
            };
        }
        const tag = tags[0]!;
        if (tag.code === DiagnosticCode.GlobalVariable || tag.code === DiagnosticCode.GlobalDynamicAccess) {
            return {
                range: tag.range,
                text: model.getValueInRange(tag.range) ?? '',
                rejectReason: 'Cannot rename global variables',
            };
        }
        return {
            range: tag.range,
            text: model.getValueInRange(tag.range) ?? '',
        };
    }
}

languages.registerRenameProvider('mirascript', new RenameProvider());
