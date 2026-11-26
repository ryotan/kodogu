import path from 'node:path';
import {fileURLToPath} from 'node:url';

import react from '@vitejs/plugin-react';
import {defineConfig} from 'vite';

import router from '@tanstack/router-plugin/vite';

const VITE_DEFAULT_PORT = 5173;

const dirname = path.dirname(fileURLToPath(import.meta.url));

const {TAURI_DEV_HOST: host, TAURI_DEV_SERVER_PORT: portString} = process.env;

// Configurations for mobile app development.
// Tauri CLI adds environment variables to the dev-server process when running Android app.
const port =
  portString && Number.isSafeInteger(Number.parseInt(portString, 10))
    ? Number.parseInt(portString, 10)
    : VITE_DEFAULT_PORT;

// The route definitions are in the package `routing`, outside this package and outside Vite's root.
const routesDirectory = path.resolve(dirname, '../routing/src/routes');

// https://vitejs.dev/config/
export default defineConfig(() => ({
  plugins: [
    router({
      target: 'react',
      autoCodeSplitting: true,
      routesDirectory,
      generatedRouteTree: path.resolve(dirname, '../routing/gen/router/routes.ts'),
      routeFileIgnorePattern: '\\.(spec|test|story)\\.[tj]sx?$',
    }),
    react(),
    {
      // The routes directory is outside Vite's root, so the watcher does not see new files in it.
      name: 'watch-routes-directory',
      configureServer(server) {
        server.watcher.add(routesDirectory);
      },
    },
  ],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: port,
    strictPort: true,
    host: host ?? false,
    // hmr host and port of ws client in mobile apps is not set by default, so we need to set it manually
    hmr: host ? {host, port} : undefined,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ['**/src-tauri/**'],
    },
  },
}));
