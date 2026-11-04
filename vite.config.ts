import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { fileURLToPath, URL } from 'node:url';

// Tauri expects a fixed port and does not want Vite to obscure Rust errors.
const host = process.env['TAURI_DEV_HOST'];

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
    watch: {
      // Rust sources are rebuilt by Cargo, not Vite.
      ignored: ['**/src-tauri/**', '**/crates/**'],
    },
  },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: {
    // Tauri uses Chromium on Windows and WebKit on macOS and Linux.
    target: process.env['TAURI_ENV_PLATFORM'] === 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env['TAURI_ENV_DEBUG'] ? 'esbuild' : false,
    sourcemap: !!process.env['TAURI_ENV_DEBUG'],
    outDir: 'dist',
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.{test,spec}.{ts,tsx}'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'lcov'],
      include: ['src/**/*.{ts,tsx}'],
      exclude: ['src/types/generated', 'src/**/*.d.ts'],
    },
  },
});
