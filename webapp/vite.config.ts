import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

const gitHash = (process.env.GIT_HASH ?? 'unknown').slice(0, 10)
const gitCommittedAt = process.env.GIT_COMMITTED_AT ?? 'unknown'

// API_PORT/WEB_PORT let multiple git worktrees run this stack at once on
// distinct ports — see ../dev.sh, which sets them automatically.
const apiPort = process.env.API_PORT ?? '3000'
const webPort = process.env.WEB_PORT ? Number(process.env.WEB_PORT) : undefined

// https://vite.dev/config/
export default defineConfig({
  define: {
    __GIT_HASH__: JSON.stringify(gitHash),
    __GIT_COMMITTED_AT__: JSON.stringify(gitCommittedAt),
  },
  plugins: [react()],
  server: {
    port: webPort,
    proxy: {
      '/api': {
        target: `http://localhost:${apiPort}`,
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, ''),
      },
    },
  },
})
