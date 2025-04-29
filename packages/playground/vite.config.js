import { defineConfig } from 'vite';
import checker from 'vite-plugin-checker';

// https://vitejs.dev/config/
export default defineConfig({
    plugins: [checker({ typescript: true })],
    build: {
        target: 'esnext',
        sourcemap: true,
        minify: false,
        manifest: false,
    },
});
