import { createRouter, createWebHashHistory } from 'vue-router'
import ScanPage from '../pages/ScanPage.vue'
import ResultsPage from '../pages/ResultsPage.vue'
import HistoryPage from '../pages/HistoryPage.vue'
import WhitelistPage from '../pages/WhitelistPage.vue'

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
  },
  {
    path: '/history',
    component: HistoryPage,
    name: 'History'
  },
  {
    path: '/whitelist',
    component: WhitelistPage,
    name: 'Whitelist'
  }
]

const router = createRouter({
  history: createWebHashHistory(),
  routes
})

export default router
