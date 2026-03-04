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
    #globalModel?: editor.ITextModel;
    /** 全局模型 */
    protected async getGlobalModel(): Promise<editor.ITextModel | undefined> {
        if (!this.#globalModel || this.#globalModel.isDisposed()) {
            this.#globalModel = (await this.createGlobalModel('mirascript:///lib/global.mira')) ?? undefined;
        }
        return this.#globalModel;
    }
    /** 创建全局模型 */
    createGlobalModel = (uri: string): languages.ProviderResult<editor.ITextModel> =>
        editor.createModel(``, 'mirascript-doc', Uri.parse(uri));
    /** 准备要显示的定义 */
    private async prepareGlobal(
        model: editor.ITextModel,
        paths: ReadonlyArray<readonly string[]>,
        focus = 0,
    ): Promise<{ uri: Uri; range: IRange } | undefined> {
        const globalModel = await this.getGlobalModel();
        if (!globalModel) return undefined;
        const globals = await this.getContext(model);
        const code: string[] = [];
        const focusRange: Writable<IRange> = {
            startColumn: 1,
            startLineNumber: 1,
            endColumn: 1,
            endLineNumber: 1,
        };
        for (const [index, path] of paths.entries()) {
            const [name, ...access] = path;
            const [parent, value] = getDeep(globals, name!, access);
            if (value === undefined) return undefined;
            const { script, doc } = valueDoc(path.at(-1)!, value, 'declare', parent);
            if (index !== focus) {
                code.push(...docComment(doc), ...script.split('\n'), '');
                continue;
            }
            code.push(...docComment(doc));
            const defName = path.at(-1)!;
            const scriptLines = script.split('\n');
            const decLine = scriptLines.findIndex((line) => line.includes(defName));
            if (decLine < 0) {
                focusRange.startLineNumber = code.length + 1;
                code.push(...script.split('\n'));
                focusRange.endLineNumber = code.length + 1;
            } else {
                code.push(...script.split('\n'));
                const decLineText = scriptLines[decLine]!;
                focusRange.startLineNumber = code.length - scriptLines.length + decLine + 1;
                focusRange.endLineNumber = focusRange.startLineNumber;
                const nameIndex = decLineText.indexOf(defName);
                focusRange.startColumn = nameIndex + 1;
                focusRange.endColumn = nameIndex + defName.length + 1;
            }
            code.push('');
        }
        globalModel.setValue(code.join('\n'));
        return {
            uri: globalModel.uri,
            range: focusRange,
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
                link = await this.prepareGlobal(model, [[def.name]]);
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
                const link: languages.LocationLink | undefined = await this.prepareGlobal(model, [
                    [def.name, ...fields],
                ]);
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
                    const link = await this.prepareGlobal(model, [[def.name]]);
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
