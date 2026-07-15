import { REG_IDENTIFIER_FULL } from '@mirascript/constants';
import type { Type } from './parser.js';

/** Precedence levels for parenthesization. */
const PREC_UNION = 1;
const PREC_INTERSECTION = 2;

/** Escape a string literal value for use in double-quoted strings. */
function escapeString(value: string): string {
    return value
        .replaceAll('\\', '\\\\')
        .replaceAll('"', String.raw`\"`)
        .replaceAll('\n', String.raw`\n`)
        .replaceAll('\r', String.raw`\r`)
        .replaceAll('\t', String.raw`\t`)
        .replaceAll('$', String.raw`\$`);
}

/**
 * Returns the precedence level of a type node.
 * Higher number = tighter binding.
 */
function getPrecedence(type: Type): number {
    if (typeof type === 'string' || typeof type === 'symbol') return 0;
    if (type.kind === 'union') return PREC_UNION;
    if (type.kind === 'intersection') return PREC_INTERSECTION;
    return 0;
}

/**
 * Stringify a type, adding parentheses if the child's precedence
 * is lower than the parent's required precedence.
 */
function stringifyImpl(type: Type, parentPrecedence: number): string {
    // ---- primitives ----
    if (typeof type === 'symbol') {
        return type.description ?? '';
    }
    if (typeof type === 'string') {
        return type;
    }

    // ---- compound types ----
    let result: string;

    switch (type.kind) {
        case 'literal': {
            if (typeof type.value === 'boolean') {
                return String(type.value);
            } else {
                return `"${escapeString(type.value)}"`;
            }
        }

        case 'template': {
            let tpl = '`';
            for (const part of type.parts) {
                if (typeof part === 'object' && part.kind === 'literal' && typeof part.value === 'string') {
                    tpl += part.value.replaceAll('`', '\\`').replaceAll('$', String.raw`\$`);
                } else {
                    tpl += `$(${stringifyImpl(part, 0)})`;
                }
            }
            tpl += '`';
            return tpl;
        }

        case 'array': {
            const inner = stringifyImpl(type.element, 0);
            // Use postfix notation for simple types, generic for complex
            if (typeof type.element === 'string' || typeof type.element === 'symbol') {
                result = `${inner}[]`;
            } else if (type.element.kind === 'literal' || type.element.kind === 'reflection') {
                result = `${inner}[]`;
            } else {
                return `array<${inner}>`;
            }
            break;
        }

        case 'union': {
            const parts = type.types.map((t) => stringifyImpl(t, PREC_UNION));
            result = parts.join(' | ');
            if (parentPrecedence > PREC_UNION) return `(${result})`;
            return result;
        }

        case 'intersection': {
            const parts = type.types.map((t) => stringifyImpl(t, PREC_INTERSECTION));
            result = parts.join(' & ');
            if (parentPrecedence > PREC_INTERSECTION) return `(${result})`;
            return result;
        }

        case 'record': {
            if ('fields' in type) {
                if (type.fields.length === 0) {
                    return '()';
                }
                // Check if all fields are anonymous (name === positional index)
                const allAnonymous = type.fields.every((f, i) => f.name === String(i) && !f.optional);
                if (allAnonymous) {
                    const types = type.fields.map((f) => stringifyImpl(f.type, 0));
                    // Single anonymous field needs trailing comma to distinguish from grouping
                    const trailing = type.fields.length === 1 ? ',' : '';
                    return `(${types.join(', ')}${trailing})`;
                } else {
                    const fields = type.fields.map((f) => {
                        const name = REG_IDENTIFIER_FULL.test(f.name) ? f.name : `"${escapeString(f.name)}"`;
                        const colon = f.optional ? '?:' : ':';
                        const typeStr = stringifyImpl(f.type, 0);
                        return `${name}${colon} ${typeStr}`;
                    });
                    return `(${fields.join(', ')})`;
                }
            }

            // Generic record
            if (type.key == null) {
                return `record<${stringifyImpl(type.value, 0)}>`;
            } else {
                return `record<${stringifyImpl(type.key, 0)}, ${stringifyImpl(type.value, 0)}>`;
            }
        }

        case 'function': {
            let fn = 'fn';
            if (type.name != null) fn += ` ${type.name}`;
            if (type.typeParams != null && type.typeParams.length > 0) {
                fn += `<${type.typeParams.map((t) => stringifyImpl(t, 0)).join(', ')}>`;
            }
            const params = type.params.map((p) => {
                const spread = p.spread ? '..' : '';
                const name = p.name || (p.spread ? '' : '_');
                const typeStr = p.type === 'any' && !p.spread ? '' : `: ${stringifyImpl(p.type, 0)}`;
                return `${spread}${name}${typeStr}`;
            });
            fn += `(${params.join(', ')})`;
            if (type.returns != null) {
                const ret = stringifyImpl(type.returns, 0);
                const needsParens =
                    typeof type.returns === 'object' &&
                    (type.returns.kind === 'union' || type.returns.kind === 'intersection');
                fn += ` -> ${needsParens ? `(${ret})` : ret}`;
            }
            result = fn;
            // Function types need parens inside union / intersection
            if (parentPrecedence > 0) return `(${result})`;
            return result;
        }

        case 'tuple': {
            if (type.elements.length === 0) {
                result = '[]';
            } else {
                const elements = type.elements.map((e) => {
                    const spread = e.spread ? '..' : '';
                    return `${spread}${stringifyImpl(e.type, 0)}`;
                });
                result = `[${elements.join(', ')}]`;
            }
            break;
        }

        case 'reflection': {
            result = `type(${type.name})`;
            break;
        }

        /* c8 ignore next 3 */
        default:
            (type) satisfies never;
            return 'unknown';
    }

    // Wrap in parens if needed (for non-union/intersection types that
    // appear inside higher-precedence context, e.g. function in union)
    const childPrec = getPrecedence(type);
    if (parentPrecedence > childPrec) {
        return `(${result})`;
    }
    return result;
}

/**
 * Converts a Type AST back into a MiraScript type string.
 * The output is guaranteed to be parseable by {@link parse}.
 */
export function stringify(type: Type): string {
    return stringifyImpl(type, 0);
}
