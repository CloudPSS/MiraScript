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
    favicon: 'favicon.svg',

    baseUrl: '/',
    trailingSlash: true,

    onBrokenLinks: 'throw',
    onBrokenAnchors: 'throw',
    onDuplicateRoutes: 'throw',

    i18n: { defaultLocale: 'zh-hans', locales: ['zh-hans'] },

    future: {
        v4: true,
        faster: true,
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
                            {
                                test: /\.node$/,
                                type: 'asset/resource',
                            },
                            {
                                test: /\.(mira|miratpl)$/,
                                type: 'asset/source',
                            },
                        ],
                    },
                    output: {
                        assetModuleFilename: 'assets/[hash][ext][query]',
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

                    showLastUpdateTime: true,
                    editUrl: 'https://github.com/CloudPSS/MiraScript/edit/main/packages/website',
                },
                theme: {
                    customCss: './src/css/index.css',
                },
            } satisfies import('@docusaurus/preset-classic').Options,
        ],
    ],
    clientModules: ['./src/js/index.ts'],
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
                    href: '/playground/',
                    label: '在线编辑器',
                },
                {
                    href: 'https://github.com/CloudPSS/MiraScript',
                    position: 'right',
                    className: 'navbar__github-link',
                    title: 'GitHub',
                },
            ],
            logo: {
                src: 'favicon.svg',
                alt: 'MiraScript Logo',
                width: 32,
                height: 32,
            },
        },
        prism: {
            theme: prismThemes.vsDark,
            // darkTheme: prismThemes.vsDark,
        },
    } satisfies import('@docusaurus/preset-classic').ThemeConfig,
} satisfies Config;
