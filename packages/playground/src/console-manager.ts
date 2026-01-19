import type { VmAny } from '@mirascript/mirascript';
import { escapeHtml, print } from './utils.js';
import { mirascriptSubtle } from './loader.js';

/** 消息 */
type Message = VmAny[] | string | Error;
/** 级别 */
type Level = 'log' | 'error' | 'warn' | 'info';
const background: Record<Level, string> = {
    log: 'background: #1177bb; color: #fff;',
    error: 'background: #d23d3d; color: #fff;',
    warn: 'background: #c39c00; color: #fff;',
    info: 'background: #369481; color: #fff;',
};

/** 默认渲染 */
async function printValue(arg: VmAny): Promise<string> {
    if (typeof arg == 'string') return escapeHtml(arg);
    return print(arg);
}
/** 渲染 debug_print 的调用 */
async function renderDebugPrint(args: VmAny[]): Promise<string> {
    if (args.length === 0) {
        return '';
    }
    if (args.length <= 1 || typeof args[0] != 'string' || !args[0].includes('%')) {
        return (await Promise.all(args.map(printValue))).join(' ');
    }

    const { toString, toNumber } = mirascriptSubtle.convert;
    const [format, ...values] = args;
    const parts = format.split(/(%[%sdifoOc])/g);
    const rendered = [];
    let valIndex = 0;
    for (let i = 0; i < parts.length; i++) {
        if (i % 2 === 0) {
            // Regular string part
            rendered.push(escapeHtml(parts[i]!));
            continue;
        }
        // Specifier part
        const specifier = parts[i]!;
        if (specifier === '%%') {
            rendered.push('%');
        } else {
            if (valIndex >= values.length) {
                rendered.push(specifier);
                continue;
            }
            const arg = values[valIndex++]!;
            let formatted: string | { style: string } | Promise<string>;
            switch (specifier) {
                case '%s':
                    formatted = escapeHtml(toString(arg));
                    break;
                case '%f':
                case '%d':
                    formatted = escapeHtml(toString(toNumber(arg, Number.NaN)));
                    break;
                case '%i':
                    formatted = escapeHtml(toString(Math.trunc(toNumber(arg, Number.NaN))));
                    break;
                case '%c':
                    formatted = { style: toString(arg) };
                    break;
                default:
                    formatted = print(arg);
                    break;
            }
            rendered.push(formatted);
        }
    }

    // Append any remaining arguments separated by spaces
    if (valIndex < values.length) {
        const remaining = values.slice(valIndex);
        rendered.push(...remaining.map(async (v) => ' ' + (await printValue(v))));
    }
    let currentStyle = '';
    let final = '';
    // eslint-disable-next-line @typescript-eslint/await-thenable
    for (const part of await Promise.all(rendered)) {
        if (typeof part === 'string') {
            if (currentStyle) {
                final += `<span style="${currentStyle}">${part}</span>`;
            } else {
                final += part;
            }
        } else {
            currentStyle = part.style;
        }
    }
    return final;
}

/** 管理控制台输出的类 */
export class ConsoleManager {
    private entries: Array<{
        type: Level;
        message: Message;
        timestamp: number;
    }> = [];
    private readonly outputElement: HTMLElement;

    constructor(outputElement: HTMLElement) {
        this.outputElement = outputElement;
    }

    private t0 = 0;
    private d0 = 0;

    /** 开始计时 */
    resetTimer(): void {
        this.t0 = performance.now();
        this.d0 = Date.now();
    }

    /** 添加日志消息 */
    log(message: Message): void {
        this.addEntry('log', message);
    }

    /** 添加错误消息 */
    error(message: Message): void {
        this.addEntry('error', message);
    }

    /** 添加警告消息 */
    warn(message: Message): void {
        this.addEntry('warn', message);
    }

    /** 添加信息消息 */
    info(message: Message): void {
        this.addEntry('info', message);
    }

    /** 添加条目到控制台 */
    private addEntry(type: Level, message: Message): void {
        const entry = {
            type,
            message,
            timestamp: performance.now() - this.t0,
        };
        this.entries.push(entry);
        if (Array.isArray(message)) {
            mirascriptSubtle.lib.debug_print(...message);
        } else {
            // eslint-disable-next-line no-console
            console[type](
                `%cMiraScript Playground`,
                `display: inline-block; border-radius: 3px; padding: 1px 4px; ${background[type]}`,
                message,
            );
        }
    }

    /** 清空控制台 */
    clear(): void {
        this.entries = [];
    }
    /** 渲染控制台内容 */
    async render(): Promise<void> {
        const maxWidth = (this.entries.at(-1)?.timestamp ?? 0).toFixed(3).length + 1;
        const htmlArray = this.entries.map(async (entry) => {
            const { timestamp, message, type } = entry;
            const time = `${('+' + timestamp.toFixed(3)).padStart(maxWidth)}ms`;
            const date = new Date(this.d0 + timestamp);
            let rendered;
            if (typeof message == 'string') {
                rendered = escapeHtml(message);
            } else if (Array.isArray(message)) {
                rendered = await renderDebugPrint(message);
            } else {
                rendered = escapeHtml(message.message);
            }
            return /* html */ `<div class="console-entry ${type}">
                <time class="console-time" datetime="${date.toISOString()}">${time}</time>
                <span class="console-message">${rendered}</span>
            </div>`;
        });

        const html = await Promise.all(htmlArray).then((lines) => lines.join(''));
        this.outputElement.innerHTML = html;
        this.outputElement.scrollTop = this.outputElement.scrollHeight;
    }
}
