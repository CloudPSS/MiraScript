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
import { getGlobal } from './utils';
import { DOC_HEADER } from './constants';

const globalModel = editor.createModel(``, 'mirascript', Uri.parse('mirascript:///lib/global.mira'));
const prepareGlobal = (name: string): { uri: Uri; range: IRange } => {
    const { script, doc } = getGlobal(name);
    const code = [`/**${DOC_HEADER}**/`, '', `/**`, ...doc.split('\n').map((line) => ` * ${line}`), ` */`, script, ''];
    globalModel.setValue(code.join('\n'));
    const word = globalModel.getWordAtPosition({ lineNumber: code.length - 1, column: 1 });
    return {
        uri: globalModel.uri,
        range: {
            startColumn: word ? word.startColumn : 1,
            startLineNumber: code.length - 1,
            endColumn: word ? word.endColumn : 1,
            endLineNumber: code.length - 1,
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
        const { def, ref } = d;
        let originSelectionRange;
        if (ref != null) {
            originSelectionRange = def.references[ref]?.range;
        } else if ('definition' in def) {
            originSelectionRange = def.definition.range;
        }
        let link: languages.LocationLink;
        if ('name' in def) {
            link = prepareGlobal(def.name);
        } else {
            link = { uri: model.uri, range: def.definition.range };
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
            if ('name' in def) {
                links.push(prepareGlobal(def.name));
            } else if (!Range.isEmpty(def.definition.range)) {
                links.push({
                    uri: model.uri,
                    range: def.definition.range,
                });
            }
        }
        return links;
    }
}

const instance = new DefinitionReferenceProvider();
languages.registerDefinitionProvider('mirascript', instance);
languages.registerReferenceProvider('mirascript', instance);
