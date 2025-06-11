import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';
import checker from 'vite-plugin-checker';

const PROD = process.env.NODE_ENV === 'production';

// https://vitejs.dev/config/
export default defineConfig({
    base: '',
    appType: 'mpa',
    optimizeDeps: {
        exclude: ['@private/monaco-editor'],
    },
    resolve: {
        conditions: PROD ? undefined : ['module', 'browser', 'development|production', '@mira/development'],
    },
    plugins: [checker({ typescript: true }), wasm()],
    worker: {
        format: 'es',
        plugins: () => [wasm()],
    },
    build: {
        target: 'esnext',
        sourcemap: true,
        minify: false,
        manifest: false,
    },
});
