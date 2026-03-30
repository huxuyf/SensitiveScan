import { createRouter, createWebHashHistory } from 'vue-router'
import ScanPage from './ScanPage.vue'
import ResultsPage from './ResultsPage.vue'
import HistoryPage from './HistoryPage.vue'
import WhitelistPage from './WhitelistPage.vue'

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
