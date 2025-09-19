import { defineConfig } from 'vite';
import checker from 'vite-plugin-checker';
import sourcemaps from 'rollup-plugin-sourcemaps2';

const PROD = process.env.NODE_ENV === 'production';

// https://vitejs.dev/config/
export default defineConfig({
    base: './',
    appType: 'spa',
    optimizeDeps: {
        exclude: ['@private/monaco-editor'],
    },
    resolve: {
        conditions: PROD ? undefined : ['module', 'browser', 'development|production', '@mirascript/dev'],
    },
    plugins: [sourcemaps(), checker({ typescript: true })],
    worker: {
        format: 'es',
    },
    css: {
        transformer: 'lightningcss',
        devSourcemap: true,
    },
    server: {
        headers: {
            'Cross-Origin-Opener-Policy': 'same-origin',
            'Cross-Origin-Embedder-Policy': 'require-corp',
        },
    },
    build: {
        cssMinify: 'lightningcss',
        sourcemap: true,
        emptyOutDir: true,
    },
});
