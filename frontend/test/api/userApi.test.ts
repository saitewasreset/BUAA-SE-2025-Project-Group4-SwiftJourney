import { userApi } from '../../src/api/UserApi/userApi';
import { describe, it, expect } from 'vitest';

describe('userApi', () => {
    it('userLogin should return a promise', () => {
        const result = userApi.userLogin({});
        expect(result).toBeInstanceOf(Promise);
    });

    it('setUserInfo should return a promise', () => {
        const result = userApi.setUserInfo({});
        expect(result).toBeInstanceOf(Promise);
    });

    it('getUserInfo should return a promise', () => {
        const result = userApi.getUserInfo();
        expect(result).toBeInstanceOf(Promise);
    });

    it('updatePassword should return a promise', () => {
        const result = userApi.updatePassword({});
        expect(result).toBeInstanceOf(Promise);
    });

    it('userRegister should return a promise', () => {
        const result = userApi.userRegister({});
        expect(result).toBeInstanceOf(Promise);
    });

    it('userLogout should return a promise', () => {
        const result = userApi.userLogout();
        expect(result).toBeInstanceOf(Promise);
    });

    it('queryUserBalance should return a promise', () => {
        const result = userApi.queryUserBalance();
        expect(result).toBeInstanceOf(Promise);
    });

    it('queryPersonalInfo should return a promise', () => {
        const result = userApi.queryPersonalInfo();
        expect(result).toBeInstanceOf(Promise);
    });

    it('setPersonalInfo should return a promise', () => {
        const result = userApi.setPersonalInfo({});
        expect(result).toBeInstanceOf(Promise);
    });
});