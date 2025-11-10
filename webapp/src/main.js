import { createApp } from 'vue';
import PrimeVue from 'primevue/config';
import Aura from '@primevue/themes/aura';
import App from './App.vue';

// Prime Icons
import 'primeicons/primeicons.css';

const app = createApp(App);
app.use(PrimeVue, {
  theme: {
    preset: Aura,
    options: {
      darkModeSelector: 'system' // respects system preference
    }
  }
});

app.mount('#app');
