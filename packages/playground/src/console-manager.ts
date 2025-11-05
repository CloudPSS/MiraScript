import type { VmAny } from '@mirascript/mirascript';
import { escapeHtml, print } from './utils.js';
import { lib } from '@mirascript/mirascript/subtle';

/** 消息 */
type Message = readonly VmAny[] | string;
/** 管理控制台输出的类 */
export class ConsoleManager {
    private entries: Array<{
        type: 'log' | 'error' | 'warn' | 'info';
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
    private addEntry(type: 'log' | 'error' | 'warn' | 'info', message: Message): void {
        const entry = {
            type,
            message,
            timestamp: performance.now() - this.t0,
        };
        this.entries.push(entry);
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
            } else {
                lib.debug_print(...message);
                rendered = (
                    await Promise.all(
                        message.map(async (arg) => {
                            if (typeof arg == 'string') return escapeHtml(arg);
                            return print(arg);
                        }),
                    )
                ).join(' ');
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
