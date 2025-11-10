# WebApp (Vite + Vue 3 + PrimeVue Aura)

A minimal single-page web interface using Vite, Vue 3, and PrimeVue with the Aura theme. It renders a centered stack of PrimeVue `Card` components.

## Prerequisites
- Node.js 18+ (recommended: 20 LTS)
- npm 9+ or pnpm/yarn

## Getting started
```bash
cd webapp
npm install
npm run dev
```
Then open the URL printed by Vite (usually http://localhost:5173/). The page shows a vertically stacked set of cards centered in the viewport.

## Build for production
```bash
npm run build
# Preview the production build
npm run preview
```

## Notes
- PrimeVue is configured with the Aura theme in `src/main.js`.
- The UI is currently static. To integrate with the Rust backend, expose an HTTP API (e.g., with Axum or Actix) and fetch data in `App.vue`.
