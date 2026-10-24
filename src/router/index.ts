import { createRouter, createWebHashHistory } from 'vue-router'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', redirect: '/today' },
    { path: '/today', component: () => import('../views/TodayView.vue') },
    { path: '/tasks', component: () => import('../views/TasksView.vue') },
    { path: '/trajectory', component: () => import('../views/TrajectoryView.vue') },
    { path: '/projects', component: () => import('../views/ProjectsView.vue') },
    { path: '/ddl', component: () => import('../views/DdlView.vue') },
    { path: '/weekly', component: () => import('../views/WeeklyView.vue') },
    { path: '/review', component: () => import('../views/ReviewView.vue') },
    { path: '/papers', component: () => import('../views/PapersView.vue') },
    { path: '/achievements', component: () => import('../views/AchievementsView.vue') },
    { path: '/expenses', component: () => import('../views/ExpensesView.vue') },
    { path: '/settings', component: () => import('../views/SettingsView.vue') },
  ],
})

export default router
