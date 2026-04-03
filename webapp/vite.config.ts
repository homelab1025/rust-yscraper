import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { execSync } from 'child_process'

const git = (args: string) => {
  try {
    return execSync(`git ${args}`).toString().trim()
  } catch {
    return 'unknown'
  }
}

const gitHash = git('rev-parse HEAD').slice(0, 10)
const gitCommittedAt = git('log -1 --format=%cI HEAD')

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
