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
import { getGlobalScript } from './utils';
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
        const d = compiled.definition(model, position);
        if (!d) return [];
        const { range, references, definition } = d.def;
        if (!range) return [];
        let originSelectionRange;
        if (d.ref == null) {
            originSelectionRange = definition?.range;
        } else {
            originSelectionRange = references[d.ref]?.range;
        }
        let link: languages.LocationLink;
        if (typeof range == 'string') {
            link = prepareGlobal(range);
        } else {
            link = { uri: model.uri, range: range };
        }
        link.originSelectionRange = originSelectionRange;
        return [link];
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
        const d = compiled.definition(model, position);
        if (!d) return [];
        const { def } = d;
        const links: languages.Location[] = def.references.map((u) => ({
            uri: model.uri,
            range: u.range,
        }));
        if (context.includeDeclaration) {
            if (def.definition && !Range.isEmpty(def.definition.range)) {
                links.push({
                    uri: model.uri,
                    range: def.definition.range,
                });
            } else if (typeof def.range == 'string' && def.range) {
                links.push(prepareGlobal(def.range));
            }
        }
        return links;
    }
}

const instance = new DefinitionReferenceProvider();
languages.registerDefinitionProvider('mirascript', instance);
languages.registerReferenceProvider('mirascript', instance);
