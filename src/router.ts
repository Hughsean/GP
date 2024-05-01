import { createRouter, createWebHashHistory } from 'vue-router'

import Init from './components/Init.vue'
import Start from './components/Start.vue'
import Play from './components/Play.vue'

const routes = [
    {
        path: '/',
        component: Init
    },
    {
        path: '/Start',
        component: Start
    }, {
        path: '/Play',
        component: Play
    }
]

const router = createRouter({
    history: createWebHashHistory(),
    routes
})

export default router
