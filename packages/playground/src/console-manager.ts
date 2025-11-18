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
                rendered = message;
            } else if (Array.isArray(message)) {
                rendered = (
                    await Promise.all(
                        message.map(async (arg) => {
                            if (typeof arg == 'string') return escapeHtml(arg);
                            return print(arg);
                        }),
                    )
                ).join(' ');
            } else {
                rendered = message.message;
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
