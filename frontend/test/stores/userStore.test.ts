import { describe, it, expect, beforeEach, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useUserStore, useDebugUserStore } from '@/stores/user';
import { userApi } from '@/api/UserApi/userApi';
import { message } from 'ant-design-vue';

// Mock dependencies
vi.mock('@/api/UserApi/userApi');
vi.mock('ant-design-vue', () => ({
    message: {
        error: vi.fn(),
        success: vi.fn()
    }
}));

// Mock localStorage
const localStorageMock = {
    getItem: vi.fn(),
    setItem: vi.fn(),
    removeItem: vi.fn()
};
Object.defineProperty(window, 'localStorage', {
    value: localStorageMock
});

describe('useUserStore', () => {
    let userStore: ReturnType<typeof useUserStore>;
    const mockRouter = { push: vi.fn() };

    beforeEach(() => {
        setActivePinia(createPinia());
        userStore = useUserStore();
        vi.clearAllMocks();
    });

    describe('initial state', () => {
        it('should have correct initial state', () => {
            expect(userStore.personalId).toBe('');
            expect(userStore.username).toBe('');
            expect(userStore.name).toBe('');
            expect(userStore.identityCardId).toBe('');
            expect(userStore.preferredSeatLocation).toBeUndefined();
            expect(userStore.gender).toBe('male');
            expect(userStore.age).toBeUndefined();
            expect(userStore.phone).toBe('');
            expect(userStore.email).toBe('');
            expect(userStore.havePaymentPasswordSet).toBe(false);
            expect(userStore.remainingMoney).toBe('SC 0');
        });
    });

    describe('getters', () => {
        it('should return true for isLogin when localStorage has isLogin as true', () => {
            localStorageMock.getItem.mockReturnValue('true');
            expect(userStore.isLogin).toBe(true);
        });

        it('should return true for isLogin when username is not empty', () => {
            localStorageMock.getItem.mockReturnValue('false');
            userStore.username = 'testuser';
            expect(userStore.isLogin).toBe(true);
        });

        it('should return false for isLogin when localStorage is false and username is empty', () => {
            localStorageMock.getItem.mockReturnValue('false');
            userStore.username = '';
            expect(userStore.isLogin).toBe(false);
        });
    });

    describe('actions', () => {
        describe('setPreferredSeatLocation', () => {
            it('should set preferred seat location', () => {
                userStore.setPreferredSeatLocation('B');
                expect(userStore.preferredSeatLocation).toBe('B');
            });
        });

        describe('setUserInfo', () => {
            it('should set user info correctly', () => {
                const userInfo = {
                    username: 'testuser',
                    gender: 'female' as const,
                    age: 25,
                    phone: '1234567890',
                    email: 'test@example.com',
                    havePaymentPasswordSet: true,
                    name: 'Test User',
                    identityCardId: '123456789012345678'
                };

                userStore.setUserInfo(userInfo);

                expect(localStorageMock.setItem).toHaveBeenCalledWith('isLogin', 'true');
                expect(userStore.username).toBe('testuser');
                expect(userStore.gender).toBe('female');
                expect(userStore.age).toBe(25);
                expect(userStore.phone).toBe('1234567890');
                expect(userStore.email).toBe('test@example.com');
                expect(userStore.havePaymentPasswordSet).toBe(true);
                expect(userStore.name).toBe('Test User');
                expect(userStore.identityCardId).toBe('123456789012345678');
            });

            it('should handle undefined values', () => {
                const userInfo = {
                    username: 'testuser',
                    gender: 'male' as const,
                    phone: '1234567890',
                    havePaymentPasswordSet: false,
                    name: 'Test User',
                    identityCardId: '123456789012345678'
                };

                userStore.setUserInfo(userInfo);

                expect(userStore.age).toBe(0);
                expect(userStore.email).toBe('');
            });
        });

        describe('setPersonalInfo', () => {
            it('should set personal info correctly', () => {
                const personalInfo = {
                    personalId: 'personal123',
                    name: 'Personal Name',
                    identityCardId: '987654321098765432',
                    preferredSeatLocation: 'C' as const
                };

                userStore.setPersonalInfo(personalInfo);

                expect(userStore.personalId).toBe('personal123');
                expect(userStore.name).toBe('Personal Name');
                expect(userStore.identityCardId).toBe('987654321098765432');
                expect(userStore.preferredSeatLocation).toBe('C');
            });
        });

        describe('setUserBalance', () => {
            it('should set user balance correctly', () => {
                userStore.setUserBalance(1500);
                expect(userStore.remainingMoney).toBe('SC 1500');
            });
        });

        describe('clearUserInfo', () => {
            it('should clear all user info and remove from localStorage', () => {
                userStore.clearUserInfo();

                expect(userStore.username).toBe('');
                expect(userStore.name).toBe('');
                expect(userStore.identityCardId).toBe('');
                expect(userStore.preferredSeatLocation).toBe('A');
                expect(userStore.gender).toBe('male');
                expect(userStore.age).toBe(0);
                expect(userStore.phone).toBe('');
                expect(userStore.email).toBe('');
                expect(userStore.havePaymentPasswordSet).toBe(false);
                expect(userStore.remainingMoney).toBe('SC 0');
                expect(localStorageMock.removeItem).toHaveBeenCalledWith('isLogin');
            });
        });

        describe('logout', () => {
            it('should handle successful logout', async () => {
                userStore.username = 'testuser';
                localStorageMock.getItem.mockReturnValue('true');
                vi.mocked(userApi.userLogout).mockResolvedValue({
                    data: { code: 200 }
                });

                await userStore.logout(mockRouter);

                expect(message.success).toHaveBeenCalledWith('登出成功');
                expect(mockRouter.push).toHaveBeenCalledWith('/login');
            });

            it('should handle logout when not logged in', async () => {
                userStore.username = '';
                localStorageMock.getItem.mockReturnValue('false');

                await userStore.logout(mockRouter);

                expect(message.error).toHaveBeenCalledWith('您尚未登录');
                expect(mockRouter.push).toHaveBeenCalledWith('/login');
            });

            it('should handle failed logout', async () => {
                userStore.username = 'testuser';
                localStorageMock.getItem.mockReturnValue('true');
                vi.mocked(userApi.userLogout).mockResolvedValue({
                    data: { code: 400 }
                });

                await userStore.logout(mockRouter);

                expect(message.error).toHaveBeenCalledWith('登录信息过期，请重新登录');
                expect(mockRouter.push).toHaveBeenCalledWith('/login');
            });
        });
    });
});

describe('useDebugUserStore', () => {
    let debugUserStore: ReturnType<typeof useDebugUserStore>;

    beforeEach(() => {
        setActivePinia(createPinia());
        debugUserStore = useDebugUserStore();
    });

    describe('initial state', () => {
        it('should have correct debug initial state', () => {
            expect(debugUserStore.personalId).toBe('123456789012345678');
            expect(debugUserStore.username).toBe('J J');
            expect(debugUserStore.name).toBe('John Doe');
            expect(debugUserStore.identityCardId).toBe('123456789012345678');
            expect(debugUserStore.preferredSeatLocation).toBe('A');
            expect(debugUserStore.isLogin).toBe(true);
            expect(debugUserStore.gender).toBe('male');
            expect(debugUserStore.age).toBe(20);
            expect(debugUserStore.phone).toBe('15338297650');
            expect(debugUserStore.email).toBe('john.doe@example.com');
            expect(debugUserStore.havePaymentPasswordSet).toBe(false);
            expect(debugUserStore.remainingMoney).toBe('SC30000');
        });
    });

    describe('actions', () => {
        it('should set preferred seat location', () => {
            debugUserStore.setPreferredSeatLocation('D');
            expect(debugUserStore.preferredSeatLocation).toBe('D');
        });

        it('should set user details', () => {
            debugUserStore.setUserDetails('newId123', 'New Name', 'newIdentity456');
            expect(debugUserStore.personalId).toBe('newId123');
            expect(debugUserStore.name).toBe('New Name');
            expect(debugUserStore.identityCardId).toBe('newIdentity456');
        });

        it('should set user login status', () => {
            debugUserStore.setUserLoginStatus(false);
            expect(debugUserStore.isLogin).toBe(false);
        });
    });
});