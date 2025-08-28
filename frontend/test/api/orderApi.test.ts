import { orderApi } from '../../src/api/orderApi/orderApi';
import { describe, it, expect } from 'vitest';


describe('orderApi', () => {
    it('orderList should return a promise', () => {
        const result = orderApi.orderList();
        expect(result).toBeInstanceOf(Promise);
    });

    it('orderCancel should return a promise', () => {
        const result = orderApi.orderCancel('123');
        expect(result).toBeInstanceOf(Promise);
    });
});
