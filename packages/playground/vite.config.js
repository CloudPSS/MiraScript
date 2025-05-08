import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';
import checker from 'vite-plugin-checker';

// https://vitejs.dev/config/
export default defineConfig({
    plugins: [checker({ typescript: true }), wasm()],
    build: {
        target: 'esnext',
        sourcemap: true,
        minify: false,
        manifest: false,
    },
});
