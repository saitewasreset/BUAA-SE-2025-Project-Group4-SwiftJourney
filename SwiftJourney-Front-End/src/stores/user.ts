import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { NormalConstants } from '@/constant/NormalConstant';
import { userApi } from '@/api/UserApi/userApi';
import { useRouter } from 'vue-router';
import Cookie from 'js-cookie';
import { message } from 'ant-design-vue';
import type { UserInfo } from '@/interface/userInterface';

const router = useRouter();

export const useUserStore = defineStore('user', {
    state: () => ({
        username: '',
        name: '',
        identityCardId: '',
        preferredSeatLocation: 'A' as 'A' | 'B' | 'C' | 'D' | 'F',
        gender: 'male' as 'male' | 'female' | undefined,
        age: 0,
        phone: '',
        email: '',
        havePaymentPasswordSet: false,
        remainingMoney: NormalConstants.RMB_SIGNAL + '0',
    }),
    getters: {
        isLogin: () => Cookie.get('session_id') !== undefined,
    },
    actions: {
        setPreferredSeatLocation(location: 'A' | 'B' | 'C' | 'D' | 'F') {
            this.preferredSeatLocation = location;
        },
        setUserInfo(userInfo: UserInfo) {
            this.username = userInfo.username;
            this.gender = userInfo.gender;
            this.age = userInfo.age !== undefined ? userInfo.age : 0;
            this.phone = userInfo.phone;
            this.email = userInfo.email !== undefined ? userInfo.email : '';
            this.havePaymentPasswordSet = userInfo.havePaymentPasswordSet;
            this.name = userInfo.name;
            this.identityCardId = userInfo.identityCardId;
        },
        clearUserInfo() {
            this.username = '';
            this.name = '';
            this.identityCardId = '';
            this.preferredSeatLocation = 'A';
            this.gender = 'male';
            this.age = 0;
            this.phone = '';
            this.email = '';
            this.havePaymentPasswordSet = false;
            this.remainingMoney = NormalConstants.RMB_SIGNAL + '0';
        },
        async restoreUserFromCookie() {
            if(!this.isLogin) {
                return;
            }
            const session_id = Cookie.get('session_id');
            await userApi.getUserInfo({ session_id: session_id }).then((res) => {
                if(res.status === 200) {
                    const userInfo: UserInfo = res.data;
                    this.setUserInfo(userInfo);
                } else {
                    message.error('获取用户信息失败，请重新登录');
                    Cookie.remove('session_id');
                    router.push('/login');
                    return;
                }
            });
            router.push('/home');
        },
        async logout() {
            if(!this.isLogin) {
                message.error('您尚未登录');
                return;
            }
            const session_id = Cookie.get('session_id');
            await userApi.userLogout({ session_id: session_id }).then((res) => {
                Cookie.remove('session_id');
                if(res.status === 200) {
                    message.success('注销成功');
                    this.clearUserInfo();
                    router.push('/home');
                } else {
                    message.error('获取用户信息失败，请重新登录');
                    router.push('/login');
                }
            });
            return;
        }
    }
});

export const useDebugUserStore = defineStore('debugUser', {
    state: () => ({
        personalId: '123456789012345678',
        username: 'J J',
        name: 'John Doe',
        identityCardId: '123456789012345678',
        preferredSeatLocation: 'A' as 'A' | 'B' | 'C' | 'D' | 'F',
        token: '',
        isLogin: true,
        gender: "male",
        age: 20,
        phone: '15338297650',
        email: 'john.doe@example.com',
        havePaymentPasswordSet: false,
        remainingMoney: NormalConstants.RMB_SIGNAL + '30000',
    }),
    getters: {

    },
    actions: {
        setPreferredSeatLocation(location: 'A' | 'B' | 'C' | 'D' | 'F') {
            this.preferredSeatLocation = location;
        },
        setUserDetails(personalId: string, name: string, identityCardId: string) {
            this.personalId = personalId;
            this.name = name;
            this.identityCardId = identityCardId;
        },
        setUserLoginStatus(isLogin: boolean) {
            this.isLogin = isLogin;
        },
        logout() {
            
        }
    }
});
