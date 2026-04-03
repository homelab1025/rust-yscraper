import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

const gitHash = (process.env.GIT_HASH ?? 'unknown').slice(0, 10)
const gitCommittedAt = process.env.GIT_COMMITTED_AT ?? 'unknown'

// https://vite.dev/config/
export default defineConfig({
  define: {
    __GIT_HASH__: JSON.stringify(gitHash),
    __GIT_COMMITTED_AT__: JSON.stringify(gitCommittedAt),
  },
  plugins: [react()],
  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, ''),
      },
    },
  },
})
