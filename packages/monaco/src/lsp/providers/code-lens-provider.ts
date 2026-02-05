import { DiagnosticCode } from '@mirascript/constants';
import { Range, type CancellationToken, type editor, type languages } from '../../monaco-api.js';
import { Provider } from './base.js';

/**
 * Code Lens
 */
export class CodeLensProvider extends Provider implements languages.CodeLensProvider {
    /** @inheritdoc */
    async provideCodeLenses(
        model: editor.ITextModel,
        token: CancellationToken,
    ): Promise<languages.CodeLensList | undefined> {
        const result = await this.getCompileResult(model);
        if (!result) return undefined;

        const lenses: languages.CodeLens[] = [];
        for (const { definition, references } of result.groupedTags(model).locals) {
            if (definition.code === DiagnosticCode.LocalFunction || definition.code === DiagnosticCode.LocalModule) {
                lenses.push({
                    range: definition.range,
                    command: {
                        id: 'editor.action.findReferences',
                        title: `${references.length} 个引用`,
                        arguments: [model.uri, Range.getStartPosition(definition.range)],
                    },
                });
            }
        }
        return { lenses };
    }

    /** @inheritdoc */
    resolveCodeLens?(
        model: editor.ITextModel,
        codeLens: languages.CodeLens,
        token: CancellationToken,
    ): languages.ProviderResult<languages.CodeLens>;
}
