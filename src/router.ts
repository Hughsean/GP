import { createRouter, createWebHashHistory } from 'vue-router'

import Init from './components/Init.vue'
import Do from './components/Do.vue'

const routes = [
    {
        path: '/',
        name: 'Init',
        component: Init
    },
    {
        path: '/Do',
        name: 'About',
        component: Do
    }
]

const router = createRouter({
    history: createWebHashHistory(),
    routes
})

export default router
