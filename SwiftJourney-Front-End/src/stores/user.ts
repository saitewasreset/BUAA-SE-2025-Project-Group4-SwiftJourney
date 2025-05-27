import { defineStore } from 'pinia';
import { NormalConstants } from '@/constant/NormalConstant';
import { userApi } from '@/api/UserApi/userApi';
import { type Router } from 'vue-router';
import { message } from 'ant-design-vue';
import type { UserApiBalanceData, UserApiResponseData, UserInfo } from '@/interface/userInterface';

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
        isLogin: () => localStorage.getItem('isLogin') === 'true',
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
            localStorage.setItem('isLogin', 'true');
        },
        setUserBalance(balance: number) {
            this.remainingMoney = NormalConstants.RMB_SIGNAL + balance.toString();
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
            localStorage.removeItem('isLogin');
        },
        async restoreUserFromCookie(router: Router) {
            try {
                const res: UserApiResponseData = (await userApi.getUserInfo()).data;
                if(res.code === 200) {
                    const userInfo: UserInfo = res.data as UserInfo;
                    this.setUserInfo(userInfo);
                    
                }
                else
                    throw new Error('invalid session id');
                const balRes: UserApiBalanceData = (await userApi.queryUserBalance()).data;
                if(balRes.code === 200) {
                    const balance: number = balRes.data.balance;
                    this.setUserBalance(balance);
                }
                else
                    throw new Error('invalid session id');
            } catch(e: any) {
                if(e.message === 'invalid session id') {
                    if(!this.isLogin) {
                        return;
                    }
                    this.clearUserInfo();
                    message.error('登录信息过期，请重新登录');
                    router.push('/login');
                }
                else {
                    message.error('系统出现其他错误');
                    console.log(e);
                }
            }
        },
        async logout(router: Router) {
            if(!this.isLogin) {
                message.error('您尚未登录');
                router.push('/login');
                return;
            }
            const res: UserApiResponseData = (await userApi.userLogout()).data;
            if(res.code === 200) {
                message.success('登出成功');
                this.clearUserInfo();
            } else {
                this.clearUserInfo();
                message.error('登录信息过期，请重新登录');
            }
                router.push('/login');
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
