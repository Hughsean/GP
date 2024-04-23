import { createApp } from 'vue'
import ElementPlus from 'element-plus'
import './styles.css'
import 'element-plus/dist/index.css'
import App from './App.vue'
const app = createApp(App)

app.use(ElementPlus)
app.mount('#app')