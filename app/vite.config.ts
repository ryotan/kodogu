import react from '@vitejs/plugin-react';
import {defineConfig} from 'vite';
import paths from 'vite-tsconfig-paths';

import router from '@tanstack/router-plugin/vite';

const VITE_DEFAULT_PORT = 5173;

const {
  TAURI_DEV_HOST: host,
  TAURI_DEV_SERVER_PORT: portString,
  // @ts-expect-error -- process is a nodejs global
  // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
} = process.env as Record<string, string | undefined>;

// Configurations for mobile app development.
// Tauri CLI adds environment variables to the dev-server process when running Android app.
const port =
  portString && Number.isSafeInteger(Number.parseInt(portString)) ? Number.parseInt(portString) : VITE_DEFAULT_PORT;

// https://vitejs.dev/config/
export default defineConfig(() => ({
  plugins: [
    paths(),
    router({target: 'react', autoCodeSplitting: true, generatedRouteTree: './gen/router/routes.ts'}),
    react(),
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
