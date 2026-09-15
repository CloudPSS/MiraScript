import { DiagnosticCode, type SourceDiagnostic } from '@mirascript/mirascript/subtle';
import { Range, type IRange } from '../../monaco-api.js';
import type { GroupedTags } from './grouped-tags.js';
import {
    ParameterDefinitionType,
    type LocalDefinition,
    type ParameterPlaceholderType,
    type SourceScope,
} from './interfaces.js';

/** 作用域信息 */
export interface ScopeInfo {
    /** 所有作用域的列表 */
    scopes: readonly SourceScope[];
    /** 局部变量到其所在作用域的映射 */
    map: ReadonlyMap<LocalDefinition, SourceScope>;
}

/** 获取作用域信息 */
export function makeScopes(tags: GroupedTags, fullRange: IRange): ScopeInfo {
    const { locals, params, ranges } = tags;

    // 1. 提取所有 Scope 范围
    const scopes: Array<Writable<SourceScope>> = ranges
        .filter((r) => r.code === DiagnosticCode.Scope)
        .map((r) => {
            return {
                range: r.range,
                locals: [],
                params: [],
                parent: undefined,
                children: [],
            };
        });

    // 2. 按范围嵌套关系构建父子树
    for (let i = 0; i < scopes.length; i++) {
        const scopeA = scopes[i]!;
        let parent: Writable<SourceScope> | undefined;
        for (let j = 0; j < scopes.length; j++) {
            if (i === j) continue;
            const scopeB = scopes[j]!;
            // 判断 scopeA 是否被 scopeB 包含（严格包含）
            const aRange = scopeA.range;
            const bRange = scopeB.range;
            const isContained = Range.containsRange(bRange, aRange);
            if (isContained) {
                // 选择最近的父作用域
                if (!parent || Range.containsRange(parent.range, bRange)) {
                    parent = scopeB;
                }
            }
        }
        if (parent) {
            if (parent.parent === scopeA) {
                continue; // 防止作用域大小相等导致的循环引用
            }
            scopeA.parent = parent;
            (parent.children as Writable<SourceScope[]>).push(scopeA);
        }
    }

    // 3. 按 BFS 顺序排序作用域
    const root = scopes.find((s) => !s.parent);
    // 由于脚本本身就是一个作用域，所以根作用域一定存在且唯一
    if (!root) {
        // 编译失败时，创建根作用域
        return {
            scopes: [
                {
                    range: fullRange,
                    locals: [],
                    params: [],
                    parent: undefined,
                    children: [],
                },
            ],
            map: new Map(),
        };
    }
    const queue: SourceScope[] = [root];
    const sortedScopes: SourceScope[] = [];
    while (queue.length > 0) {
        const scope = queue.shift()!;
        sortedScopes.push(scope);
        (scope.children as Writable<SourceScope[]>).sort((a, b) => Range.compareRangesUsingStarts(a.range, b.range));
        for (const child of scope.children) {
            queue.push(child);
        }
    }

    // 4. 填充每个作用域的局部变量
    for (const local of locals) {
        const { range } = local.definition;
        const scope = sortedScopes.findLast((s) => Range.containsRange(s.range, range));
        if (scope) {
            (scope.locals as Writable<LocalDefinition[]>).push(local);
        }
    }
    for (const param of params) {
        const { range } = param;
        const scope = sortedScopes.findLast((s) => Range.containsRange(s.range, range));
        if (scope) {
            (scope.params as Array<SourceDiagnostic<ParameterPlaceholderType>>).push(param);
        }
    }

    // 5. 构建作用域映射
    const scopeMap = new Map<LocalDefinition, SourceScope>();
    for (const scope of sortedScopes) {
        (scope.locals as Writable<LocalDefinition[]>).sort((a, b) =>
            Range.compareRangesUsingStarts(a.definition.range, b.definition.range),
        );
        (scope.params as Array<SourceDiagnostic<ParameterPlaceholderType>>).sort((a, b) =>
            Range.compareRangesUsingStarts(a.range, b.range),
        );
        for (const local of scope.locals) {
            scopeMap.set(local, scope);
            if (local.definition.code === DiagnosticCode.LocalFunction) {
                const funcScope = scope.children.find(
                    (s) => Range.compareRangesUsingStarts(s.range, local.definition.range) > 0,
                );
                if (funcScope) {
                    const args = funcScope.locals.filter((l): l is LocalDefinition<ParameterDefinitionType> =>
                        ParameterDefinitionType.includes(l.definition.code as ParameterDefinitionType),
                    );
                    (local as Writable<LocalDefinition>).fn = {
                        scope: funcScope,
                        args,
                    };
                }
            }
        }
    }

    return {
        scopes: sortedScopes,
        map: scopeMap,
    };
}
