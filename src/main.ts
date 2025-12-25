import { createApp } from 'vue';
import { createPinia } from 'pinia';
import './style.css';
import App from './App.vue';
import { initApp } from './config/bootstrap';

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);

// Bootstrap application (loading settings, theme, last project)
initApp();

app.mount('#app');
