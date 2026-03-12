import { createRouter, createWebHistory } from 'vue-router'
import ProjectList from '../views/ProjectList.vue'
import Settings from '../views/Settings.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'projects',
      component: ProjectList
    },
    {
      path: '/settings',
      name: 'settings',
      component: Settings
    }
  ]
})

export default router