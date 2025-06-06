import {
    editor,
    languages,
    Range,
    Uri,
    type CancellationToken,
    type IRange,
    type Position,
} from '@private/monaco-editor';
import { Provider } from './worker-helper';
import { getGlobalScript, strictInRange } from './utils';
import { DOC_HEADER } from './constants';

const globalModel = editor.createModel(``, 'mirascript', Uri.parse('mirascript:///lib/global.mira'));
const prepareGlobal = (name: string): { uri: Uri; range: IRange } => {
    const { script, doc } = getGlobalScript(name);
    const code = [
        `/**${DOC_HEADER}**/`,
        '',
        `/**`,
        doc
            .split('\n')
            .map((line) => ` * ${line}`)
            .join('\n'),
        ` */`,
        script,
        '',
    ];
    globalModel.setValue(code.join('\n'));
    const word = globalModel.getWordAtPosition({ lineNumber: code.length, column: 1 });
    return {
        uri: globalModel.uri,
        range: {
            startColumn: word ? word.startColumn : 1,
            startLineNumber: code.length,
            endColumn: word ? word.endColumn : 1,
            endLineNumber: code.length,
        },
    };
};
/**
 * 转到定义/引用
 */
class DefinitionReferenceProvider
    extends Provider
    implements languages.DefinitionProvider, languages.ReferenceProvider
{
    /** @inheritdoc */
    async provideDefinition(
        model: editor.ITextModel,
        position: Position,
        token: CancellationToken,
    ): Promise<languages.LocationLink[] | undefined> {
        const compiled = await Provider.getCompileResult(model);
        if (!compiled) return undefined;
        const links: languages.LocationLink[] = [];
        for (const { references, range } of compiled.definitions(model)) {
            if (range == null) continue;
            if (typeof range != 'string' && strictInRange(range, position)) {
                links.push({
                    originSelectionRange: range,
                    uri: model.uri,
                    range: range,
                });
                continue;
            }
            const reference = references.find((u) => strictInRange(u.range, position));
            if (!reference) continue;
            let r;
            if (typeof range == 'string') {
                r = prepareGlobal(range);
            } else {
                r = { uri: model.uri, range: range };
            }
            links.push({
                originSelectionRange: reference.range,
                ...r,
            });
        }
        return links;
    }
    /** @inheritdoc */
    async provideReferences(
        model: editor.ITextModel,
        position: Position,
        context: languages.ReferenceContext,
        token: CancellationToken,
    ): Promise<languages.Location[] | undefined> {
        const compiled = await Provider.getCompileResult(model);
        if (!compiled) return undefined;
        const decl = compiled
            .definitions(model)
            .find(
                ({ definition, references }) =>
                    (definition && strictInRange(definition.range, position)) ||
                    references.some((u) => strictInRange(u.range, position)),
            );
        if (!decl) return [];
        const links: languages.Location[] = decl.references.map((u) => ({
            uri: model.uri,
            range: u.range,
        }));
        if (context.includeDeclaration) {
            if (decl.definition && !Range.isEmpty(decl.definition.range)) {
                links.push({
                    uri: model.uri,
                    range: decl.definition.range,
                });
            } else if (typeof decl.range == 'string' && decl.range) {
                links.push(prepareGlobal(decl.range));
            }
        }
        return links;
    }
}

const instance = new DefinitionReferenceProvider();
languages.registerDefinitionProvider('mirascript', instance);
languages.registerReferenceProvider('mirascript', instance);
