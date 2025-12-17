import type { Config } from '@docusaurus/types';
import remarkIns from 'remark-ins';
import remarkJoinCjkLines from 'remark-join-cjk-lines';
import { remarkExtendedTable, extendedTableHandlers } from 'remark-extended-table';
import rehypeCodeEditor from './docusaurus/code-editor';
import remarkCodeEditor from './docusaurus/code-editor';

export default {
    title: 'MiraScript',
    tagline: 'A powerful scripting language',
    url: 'https://mira.cloudpss.net',

    baseUrl: '/',
    trailingSlash: true,

    onBrokenLinks: 'throw',
    onBrokenAnchors: 'throw',
    onDuplicateRoutes: 'throw',

    markdown: {
        mermaid: true,
        hooks: {
            onBrokenMarkdownImages: 'throw',
            onBrokenMarkdownLinks: 'throw',
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
            } satisfies import('@docusaurus/preset-classic').Options,
        ],
    ],
} satisfies Config;
