import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';
import checker from 'vite-plugin-checker';

// https://vitejs.dev/config/
export default defineConfig({
    optimizeDeps: {
        exclude: ['@private/monaco-editor'],
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
