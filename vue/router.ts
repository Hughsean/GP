import { createRouter, createWebHashHistory } from 'vue-router'

import Init from './components/Init.vue'

import Call from './components/Call.vue'

import Wait from './components/Wait.vue'

const routes = [
    {
        path: '/',
        component: Init
    },
    {
        path: '/Call',
        component: Call
    },
    {
        path: '/Wait',
        component: Wait
    }
]

const router = createRouter({
    history: createWebHashHistory(),
    routes
})

export default router
