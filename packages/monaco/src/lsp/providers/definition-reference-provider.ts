import type { VmAny } from 'mirascript';
import {
    editor,
    Uri,
    type languages,
    type CancellationToken,
    type IRange,
    type Position,
    Range,
} from '../../monaco-api.js';
import { Provider } from './base.js';
import { globalDoc } from '../utils.js';
import { DOC_HEADER } from '../../constants.js';

/**
 * 转到定义/引用
 */
export class DefinitionReferenceProvider
    extends Provider
    implements languages.DefinitionProvider, languages.ReferenceProvider
{
    private readonly globalModel = editor.createModel(``, 'mirascript', Uri.parse('mirascript:///lib/global.mira'));
    /** 准备要显示的定义 */
    private prepareGlobal(name: string, value: VmAny): { uri: Uri; range: IRange } {
        const { globalModel } = this;
        const { script, doc } = globalDoc(name, value);
        const code = [
            `/**${DOC_HEADER}**/`,
            '',
            `/**`,
            ...doc.split('\n').map((line) => ` * ${line}`),
            ` */`,
            script,
            '',
        ];
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
    }
    /** @inheritdoc */
    async provideDefinition(
        model: editor.ITextModel,
        position: Position,
        token: CancellationToken,
    ): Promise<languages.LocationLink[] | undefined> {
        const compiled = await this.getCompileResult(model);
        if (!compiled) return undefined;
        const globals = await this.getGlobals(model);
        const d = compiled.definitionAt(model, position);
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
            link = this.prepareGlobal(def.name, globals[def.name]);
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
        const compiled = await this.getCompileResult(model);
        if (!compiled) return undefined;
        const globals = await this.getGlobals(model);
        const d = compiled.definitionAt(model, position);
        if (!d) return [];
        const { def } = d;
        const links: languages.Location[] = def.references.map((u) => ({
            uri: model.uri,
            range: u.range,
        }));
        if (context.includeDeclaration) {
            if ('name' in def) {
                links.push(this.prepareGlobal(def.name, globals[def.name]));
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
