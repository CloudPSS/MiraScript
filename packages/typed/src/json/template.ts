import type { JSONSchema } from 'json-schema-typed';
import { REG_NUMBER } from '@mirascript/constants';
import type { TemplateType, Type } from '../parser.js';

/** Escapes a literal string for use in a regular expression */
function escapeRegex(value: string): string {
    return value.replaceAll(/[.*+?^${}()|[\]\\]/g, String.raw`\$&`);
}

export const RE_ANY = '.*?';
const RE_NUMBER = REG_NUMBER.source;
const RE_BOOLEAN = 'true|false';

/** Converts a template interpolation part into a regex pattern fragment */
function templatePartPatternImpl(part: Type): string {
    if (typeof part == 'symbol') {
        return RE_ANY;
    }
    if (typeof part == 'string') {
        if (part === 'number') return RE_NUMBER;
        if (part === 'boolean') return RE_BOOLEAN;
        if (part === 'nil' || part === 'never') return '';
        return RE_ANY;
    }
    if (part.kind === 'literal') {
        if (typeof part.value === 'boolean') {
            return String(part.value);
        } else {
            return escapeRegex(part.value);
        }
    }
    if (part.kind === 'union') {
        const patterns = new Set(part.types.map((p) => templatePartPatternImpl(p)));
        if (patterns.has(RE_ANY)) {
            return RE_ANY;
        } else {
            const hasEmpty = patterns.delete('');
            let result = Array.from(patterns).join('|');
            if (hasEmpty) {
                result = `(${result})?`;
            }
            return result;
        }
    }
    if (part.kind === 'intersection') {
        // Regex intersection is not representable in general; keep behavior conservative.
        return RE_ANY;
    }
    return RE_ANY;
}

/** Converts a template interpolation part into a regex pattern fragment */
export function templatePartPattern(part: Type, grouping: boolean): string {
    const result = templatePartPatternImpl(part);
    if (!grouping) return result;
    if (result.startsWith('(')) return result;
    return `(${result})`;
}

/** Converts a TemplateType into a JSON Schema pattern */
export function template(type: TemplateType): JSONSchema {
    const pattern = `^${type.parts
        .map((p) => {
            if (typeof p == 'object' && p.kind === 'literal' && typeof p.value === 'string') {
                return templatePartPattern(p, false);
            }
            return templatePartPattern(p, true);
        })
        .join('')}$`;
    return { type: 'string', pattern };
}
