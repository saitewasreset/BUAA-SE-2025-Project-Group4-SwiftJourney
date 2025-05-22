import type { DefineComponent } from 'vue'
import { createRouter, createWebHistory } from 'vue-router'
import { useUserStore } from '@/stores/user'
import { message } from 'ant-design-vue';

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
      path: '/personalhomepage/:activeIndex',
      name: 'personalhomepage',
      component: () => import('../views/PersonalHomePage/PersonalHomePageView.vue'),
      props: true
    },
  ],
})

router.beforeEach((to, from, next) => {
  const isLogin: Boolean = localStorage.getItem('isLogin') === 'true';
  if (!isLogin) {
    // 未登录，跳转到登录页
    if (to.path !== '/login' && to.path !== '/register') {
      message.warning('请先登录');
      next({ path: '/login', query: { redirect: to.fullPath } });
    } else {
      next();
    }
  } else {
    if (to.path === '/login' || to.path === '/register') {
      message.info('您已登录');
      next({ path: '/homepage' });
    } else {
      next();
    }
  }
})

export default router
