import {
    type editor,
    languages,
    Range,
    type CancellationToken,
    type Position,
    type IPosition,
    type IRange,
} from '@private/monaco-editor';
import { Provider } from './worker-helper';

const inRange = (range: IRange, position: IPosition): boolean => {
    return !Range.isEmpty(range) && Range.containsPosition(range, position);
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
        for (const { definition, references } of compiled.definitions) {
            if (inRange(definition.range, position)) {
                links.push({
                    originSelectionRange: definition.range,
                    uri: model.uri,
                    range: definition.range,
                });
                continue;
            }
            const reference = references.find((u) => inRange(u.range, position));
            if (reference) {
                links.push({
                    originSelectionRange: reference.range,
                    uri: model.uri,
                    range: definition.range,
                });
            }
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
        const decl = compiled.definitions.find(
            ({ definition, references }) =>
                inRange(definition.range, position) || references.some((u) => inRange(u.range, position)),
        );
        if (!decl) return [];
        const links: languages.Location[] = decl.references.map((u) => ({
            uri: model.uri,
            range: u.range,
        }));
        if (context.includeDeclaration && !Range.isEmpty(decl.definition.range)) {
            links.push({
                uri: model.uri,
                range: decl.definition.range,
            });
        }
        return links;
    }
}

const instance = new DefinitionReferenceProvider();
languages.registerDefinitionProvider('mirascript', instance);
languages.registerReferenceProvider('mirascript', instance);
