import { createApp } from 'vue';
import { createPinia } from 'pinia';
import './style.css';
import App from './App.vue';

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);

// Bootstrap application (loading settings, theme, last project)
import('./config/bootstrap').then(({ initApp }) => initApp());

app.mount('#app');
