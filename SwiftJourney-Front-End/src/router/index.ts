import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      redirect: '/homepage',
    },
    {
      path: '/homepage',
      name: 'homepage',
      component: () => import('../views/HomePage/HomePageView.vue'),
    },
    {
      path: '/trainTicket',
      name: 'trainTicket',
      component: () => import('../views/TrainTicket/TrainTicketView.vue'),
    },
    {
      path: '/login',
      name: 'login',
      component: () => import('../views/LoginPage/LoginPageView.vue'),
    },
    {
      path: '/register',
      name: 'register',
      component: () => import('../views/RegisterPage/RegisterPageView.vue'),
    },
    {
      path: '/personalhomepage',
      name: 'personalhomepage',
      component: () => import('../views/PersonalHomePage/PersonalHomePageView.vue'),
    },
  ],
})

export default router
