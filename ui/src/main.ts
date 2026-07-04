import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import './style.css';
import { useSettingsStore } from './stores/settings';

const app = createApp(App);
const pinia = createPinia();
app.use(pinia);

useSettingsStore(pinia).init();

app.mount('#app');
