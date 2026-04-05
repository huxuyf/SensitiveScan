import { createRouter, createWebHashHistory } from 'vue-router'
import ScanPage from './pages/ScanPage.vue'
import ResultsPage from './pages/ResultsPage.vue'

const routes = [
  {
    path: '/',
    redirect: '/scan'
  },
  {
    path: '/scan',
    component: ScanPage,
    name: 'Scan'
  },
  {
    path: '/results',
    component: ResultsPage,
    name: 'Results'
  }
]

const router = createRouter({
  history: createWebHashHistory(),
  routes
})

export default router
