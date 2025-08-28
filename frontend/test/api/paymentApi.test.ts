import { paymentApi } from '../../src/api/PaymentApi/paymentApi';
import { describe, it, expect } from 'vitest';


describe('paymentApi', () => {
    it('setPaymentPassword should return a promise', () => {
        const result = paymentApi.setPaymentPassword({});
        expect(result).toBeInstanceOf(Promise);
    });

    it('recharge should return a promise', () => {
        const result = paymentApi.recharge({});
        expect(result).toBeInstanceOf(Promise);
    });

    it('pay should return a promise', () => {
        const result = paymentApi.pay('123', {});
        expect(result).toBeInstanceOf(Promise);
    });
});
