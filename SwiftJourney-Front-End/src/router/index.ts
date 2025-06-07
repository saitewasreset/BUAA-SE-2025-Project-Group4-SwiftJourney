import type { DefineComponent } from 'vue'
import { createRouter, createWebHistory } from 'vue-router'
import { useUserStore } from '@/stores/user'
import { message } from 'ant-design-vue';
import { nextTick } from 'vue'

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
      path: '/hotel',
      name: 'hotel',
      component: () => import('../views/Hotel/HotelPageView.vue'),
    },
    {
      path: '/hotel/:id',
      name: 'hotelDetail',
      component: () => import('../views/Hotel/HotelDetailPageView.vue'),
      props: true,
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
      // 添加 meta 信息来标识需要强制刷新的路由
      meta: { 
        requiresAuth: true,
        forceRefresh: true 
      }
    },
    {
      path: '/paytransaction/:transactionId',
      name: 'paypage',
      component: () => import('../views/PayPage/PayPageView.vue'),
      props: route => ({
        transactionId: route.params.transactionId,
        money: route.query.money
      })
    }
  ],
})

router.beforeEach(async (to, from, next) => {
  const isLogin: Boolean = localStorage.getItem('isLogin') === 'true';
  
  if (!isLogin) {
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
    } else if(to.path === '/personalhomepage/personaldata') {
      await useUserStore().restoreUserFromCookie(router);
      next();
    } else {
        await nextTick();
        next();
    }
  }
})

export default router
