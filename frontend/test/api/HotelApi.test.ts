import { hotelApi } from '../../src/api/HotelApi/hotelApi';
import { describe, it, expect } from 'vitest';

describe('HotelApi', () => {
    it('hotelQuery should return a promise', () => {
        const result = hotelApi.hotelQuery({});
        expect(result).toBeInstanceOf(Promise);
    });

    it('hotelInfo should return a promise', () => {
        const result = hotelApi.hotelInfo('1');
        expect(result).toBeInstanceOf(Promise);
    });

    it('hotelOrderInfo should return a promise', () => {
        const result = hotelApi.hotelOrderInfo({});
        expect(result).toBeInstanceOf(Promise);
    });

    it('hotelOrder should return a promise', () => {
        const result = hotelApi.hotelOrder({});
        expect(result).toBeInstanceOf(Promise);
    });

    it('hotelQuota should return a promise', () => {
        const result = hotelApi.hotelQuota('1');
        expect(result).toBeInstanceOf(Promise);
    });

    it('hotelComment should return a promise', () => {
        const result = hotelApi.hotelComment({});
        expect(result).toBeInstanceOf(Promise);
    });
});