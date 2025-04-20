import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      // name: 'home',
      // component: HomeView,
      redirect: '/homepage',
    },
    // {
    //   path: '/about',
    //   name: 'about',
    //   // route level code-splitting
    //   // this generates a separate chunk (About.[hash].js) for this route
    //   // which is lazy-loaded when the route is visited.
    //   component: () => import('../views/AboutView.vue'),
    // },
    {
      path: '/homepage',
      name: 'homepage',
      component: () => import('../views/HomePage/HomePageView.vue'),
    },
    {
      path: '/traintacket',
      name: 'traintacket',
      component: () => import('../views/TrainTacket/TrainTacketView.vue'),
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
  ],
})

export default router
