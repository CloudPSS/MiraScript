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
import { docComment, getDeep, valueDoc } from '../utils.js';

/**
 * 转到定义/引用
 */
export class DefinitionReferenceProvider
    extends Provider
    implements languages.DefinitionProvider, languages.ReferenceProvider
{
    constructor(
        private readonly globalModel = editor.createModel(
            ``,
            'mirascript-doc',
            Uri.parse('mirascript:///lib/global.mira'),
        ),
    ) {
        super();
    }
    /** 准备要显示的定义 */
    private async prepareGlobal(
        model: editor.ITextModel,
        path: readonly string[],
    ): Promise<{ uri: Uri; range: IRange } | undefined> {
        const { globalModel } = this;
        const globals = await this.getContext(model);
        const [name, ...access] = path;
        const value = getDeep(globals, name!, access);
        if (value === undefined) return undefined;
        const { script, doc } = valueDoc(path.at(-1)!, value, 'declare');
        const code = ['', ...docComment(doc), script, ''];
        globalModel.setValue(code.join('\n'));
        return {
            uri: globalModel.uri,
            range: {
                startColumn: 1,
                startLineNumber: code.length - 1,
                endColumn: 1,
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
        const value = await this.getValueAt(model, position);
        if (!value) return undefined;
        if ('variable' in value) {
            const { def } = value.variable;
            let link: languages.LocationLink | undefined;
            if ('name' in def) {
                link = await this.prepareGlobal(model, [def.name]);
            } else {
                link = { uri: model.uri, range: def.definition.range };
            }
            if (!link) return undefined;
            link.originSelectionRange = value.range;
            return [link];
        } else {
            const {
                def: { def },
                fields,
            } = value.fields;
            if ('name' in def) {
                const link: languages.LocationLink | undefined = await this.prepareGlobal(model, [def.name, ...fields]);
                if (!link) return undefined;
                link.originSelectionRange = value.range;
                return [link];
            }
        }
        return undefined;
    }
    /** @inheritdoc */
    async provideReferences(
        model: editor.ITextModel,
        position: Position,
        context: languages.ReferenceContext,
        token: CancellationToken,
    ): Promise<languages.Location[] | undefined> {
        const value = await this.getValueAt(model, position);
        if (!value) return undefined;
        if ('variable' in value) {
            const { def } = value.variable;
            const links: languages.Location[] = def.references.map((u) => ({
                uri: model.uri,
                range: u.range,
            }));
            if (context.includeDeclaration) {
                if ('name' in def) {
                    const link = await this.prepareGlobal(model, [def.name]);
                    if (link) links.push(link);
                } else if (!Range.isEmpty(def.definition.range)) {
                    links.push({ uri: model.uri, range: def.definition.range });
                }
            }
            return links;
        }
        return undefined;
    }
}
