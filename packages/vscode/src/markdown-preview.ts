import type { HighlighterCore, ShikiTransformer } from 'shiki/core';
import type MarkdownIt from 'markdown-it';
import type Token from 'markdown-it/lib/token.mjs';

const THEMES = {
    light: 'light-plus',
    dark: 'dark-plus',
    'hc-dark': 'github-dark-high-contrast',
    'hc-light': 'github-light-high-contrast',
} as const;

/** Preserve VS Code's source-map attributes on the Shiki-generated pre element. */
function attributesTransformer(attributes: ReadonlyArray<[string, string]>): ShikiTransformer {
    return {
        name: '@mirascript/vscode/markdown-preview-attributes',
        pre(node) {
            for (const [name, value] of attributes) {
                if (name === 'class') {
                    const current = node.properties[name];
                    const classes = Array.isArray(current)
                        ? current.map(String)
                        : typeof current === 'string'
                          ? current.split(/\s+/u)
                          : [];
                    node.properties[name] = [...classes, ...value.split(/\s+/u)];
                } else {
                    node.properties[name] = value;
                }
            }
            return node;
        },
    };
}

/**
 * Disposable Markdown preview integration returned by the highlighter factory.
 */
export class MarkdownPreview {
    private highlighter!: HighlighterCore;
    private markdownIt!: MarkdownIt;
    /**
     * Initialize the shared Shiki highlighter and install the MiraScript fence renderer on a markdown-it instance.
     */
    async initialize(): Promise<void> {
        const [
            { createHighlighterCore },
            { createJavaScriptRegexEngine },
            { mirascript, mirascriptTemplate, mirascriptDoc },
            { default: lightPlus },
            { default: darkPlus },
            { default: githubDarkHighContrast },
            { default: githubLightHighContrast },
        ] = await Promise.all([
            import('shiki/core'),
            import('shiki/engine/javascript'),
            import('@mirascript/textmate'),
            import('@shikijs/themes/light-plus'),
            import('@shikijs/themes/dark-plus'),
            import('@shikijs/themes/github-dark-high-contrast'),
            import('@shikijs/themes/github-light-high-contrast'),
        ]);
        this.highlighter = await createHighlighterCore({
            langs: [mirascript, mirascriptTemplate, mirascriptDoc],
            themes: [lightPlus, darkPlus, githubDarkHighContrast, githubLightHighContrast],
            engine: createJavaScriptRegexEngine(),
        });
    }

    /** Highlight only MiraScript fences and retain VS Code's renderer for every other language. */
    extendMarkdownIt(markdownIt: MarkdownIt): MarkdownIt {
        this.markdownIt = markdownIt;

        const originalFence = markdownIt.renderer.rules.fence;
        if (!originalFence) return markdownIt;

        markdownIt.renderer.rules.fence = (tokens, index, options, env, self) => {
            const token = tokens[index]!;
            const result = this.highlightContent(token);
            if (result) return result;
            return originalFence(tokens, index, options, env, self);
        };
        return markdownIt;
    }

    /** Highlight content using Shiki. */
    private highlightContent(token: Token): string | undefined {
        const lang = this.markdownIt.utils.unescapeAll(token.info).trim().split(/\s+/u, 1)[0]?.toLowerCase();
        if (!lang) return undefined;
        try {
            return this.highlighter.codeToHtml(token.content, {
                lang,
                themes: THEMES,
                defaultColor: false,
                transformers: [attributesTransformer(token.attrs ?? [])],
            });
        } catch {
            return undefined;
        }
    }
    /**
     * Release the shared Shiki highlighter.
     */
    dispose(): void {
        this.highlighter?.dispose();
    }
}
