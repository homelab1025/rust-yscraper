import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    port: 5174,
    proxy: {
      // Forward API calls during dev to the Rust backend
      // Example: VITE_API not set and you call "/comments" -> proxies to http://localhost:3000
      '/comments': {
        target: 'http://localhost:3000',
        changeOrigin: true,
      },
      '/scrape': {
        target: 'http://localhost:3000',
        changeOrigin: true,
      },
    },
  },
});
