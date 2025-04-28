import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { NormalConstants } from '@/constant/NormalConstant';

export const useUserStore = defineStore('user', {
    state: () => ({
        personalId: '',
        name: '',
        identityCardId: '',
        preferredSeatLocation: 'A' as 'A' | 'B' | 'C' | 'D' | 'F',
        token: '',
        isLogin: false,
        gender: "male" as "male" | "female",
        age: 0,
        phone: '',
        email: '',
        havePaymentPasswordSet: false,
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

export const useDebugUserStore = defineStore('debugUser', {
    state: () => ({
        personalId: '123456789012345678',
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
