import { defineConfig } from 'vite';
import checker from 'vite-plugin-checker';

const PROD = process.env.NODE_ENV === 'production';

// https://vitejs.dev/config/
export default defineConfig({
    base: '/mira',
    appType: 'mpa',
    optimizeDeps: {
        exclude: ['@private/monaco-editor'],
    },
    resolve: {
        conditions: PROD ? undefined : ['module', 'browser', 'development|production', '@mira/development'],
    },
    plugins: [checker({ typescript: true })],
    worker: {
        format: 'es',
    },
    server: {
        headers: {
            'Cross-Origin-Opener-Policy': 'same-origin',
            'Cross-Origin-Embedder-Policy': 'require-corp',
        },
    },
    build: {
        target: 'esnext',
        sourcemap: true,
    },
});
