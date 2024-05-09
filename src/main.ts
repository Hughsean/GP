import { createApp } from 'vue'
import './styles.css'
import 'element-plus/dist/index.css'
import App from './App.vue'
import router from './router'

// 禁止使用鼠标右键
document.body.addEventListener('contextmenu', e => {
    e.preventDefault()
})

const app = createApp(App)
app.use(router)
app.mount('#app')