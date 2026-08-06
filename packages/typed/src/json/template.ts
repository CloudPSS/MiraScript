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
export function templatePartPattern(part: Type, grouping: boolean): string {
    let result: string;
    if (typeof part === 'symbol') {
        result = RE_ANY;
    } else if (typeof part === 'string') {
        switch (part) {
            case 'number':
                result = RE_NUMBER;
                break;
            case 'boolean':
                result = RE_BOOLEAN;
                break;
            case 'nil':
            case 'never':
                result = '';
                break;
            case 'array':
            case 'record':
            case 'extern':
            case 'any':
            case 'unknown':
            case 'string':
            default:
                result = RE_ANY;
                break;
        }
    } else if (part.kind === 'literal') {
        if (typeof part.value === 'boolean') {
            result = String(part.value);
        } else {
            result = escapeRegex(part.value);
        }
    } else if (part.kind === 'union') {
        const patterns = new Set(part.types.map((p) => templatePartPattern(p, false)));
        if (patterns.has(RE_ANY)) {
            result = RE_ANY;
        } else {
            const hasEmpty = patterns.delete('');
            result = Array.from(patterns).join('|');
            if (hasEmpty) {
                result = `(${result})?`;
            }
        }
    } else if (part.kind === 'intersection') {
        // Regex intersection is not representable in general; keep behavior conservative.
        result = RE_ANY;
    } else {
        result = RE_ANY;
    }
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
