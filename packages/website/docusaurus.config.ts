import type { Config } from '@docusaurus/types';
import remarkIns from 'remark-ins';
import { themes as prismThemes } from 'prism-react-renderer';
import remarkJoinCjkLines from 'remark-join-cjk-lines';
import { remarkExtendedTable, extendedTableHandlers } from 'remark-extended-table';
import rehypeCodeEditor from './docusaurus/code-editor';
import remarkCodeEditor from './docusaurus/code-editor';

export default {
    title: 'MiraScript',
    tagline: 'A powerful scripting language',
    url: 'https://mira.cloudpss.net',
    favicon: 'img/mirascript-logo.svg',

    baseUrl: '/',
    trailingSlash: true,

    onBrokenLinks: 'throw',
    onBrokenAnchors: 'throw',
    onDuplicateRoutes: 'throw',

    future: {
        v4: true,
        experimental_faster: true,
    },

    markdown: {
        hooks: {
            onBrokenMarkdownImages: 'throw',
            onBrokenMarkdownLinks: 'throw',
        },
        remarkRehypeOptions: {
            handlers: {
                ...extendedTableHandlers,
            },
        },
    },
    plugins: [
        () => ({
            name: 'url-loader',
            configureWebpack(config) {
                return {
                    module: {
                        rules: [
                            {
                                test: /\.wasm$/,
                                resourceQuery: /url/,
                                type: 'asset/resource',
                            },
                        ],
                    },
                };
            },
        }),
        [
            '@easyops-cn/docusaurus-search-local',
            {
                docsRouteBasePath: '/',
                docsDir: '../../docs',
                indexBlog: false,
                language: ['zh', 'en'],
                hashed: 'query',
            } satisfies import('@easyops-cn/docusaurus-search-local').PluginOptions,
        ],
    ],
    presets: [
        [
            'classic',
            {
                docs: {
                    routeBasePath: '/',
                    path: '../../docs',

                    admonitions: {
                        extendDefaults: true,
                        keywords: ['summary'],
                    },
                    remarkPlugins: [remarkJoinCjkLines, remarkIns, remarkExtendedTable, remarkCodeEditor],
                    rehypePlugins: [rehypeCodeEditor],
                    beforeDefaultRemarkPlugins: [],
                    beforeDefaultRehypePlugins: [],
                },
                theme: {
                    customCss: './src/css/index.css',
                },
            } satisfies import('@docusaurus/preset-classic').Options,
        ],
    ],
    themeConfig: {
        colorMode: {
            respectPrefersColorScheme: true,
        },
        navbar: {
            title: 'MiraScript',
            items: [
                {
                    to: '/tutorial/introduction/',
                    label: '入门教程',
                },
                {
                    to: '/lib/',
                    label: 'API 文档',
                },
                {
                    to: '/cheatsheet/',
                    label: '速查手册',
                },
                {
                    href: 'https://mira.cloudpss.net/playground/',
                    label: '在线编辑器',
                },
            ],
            logo: {
                src: 'img/mirascript-logo.svg',
                alt: 'MiraScript Logo',
            },
        },
        prism: {
            theme: prismThemes.vsDark,
            // darkTheme: prismThemes.vsDark,
        },
    } satisfies import('@docusaurus/preset-classic').ThemeConfig,
} satisfies Config;
