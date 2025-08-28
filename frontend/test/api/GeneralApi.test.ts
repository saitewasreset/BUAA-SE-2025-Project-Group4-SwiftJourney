import { generalApi } from "../../src/api/GeneralApi/generalApi";
import { describe, it, expect } from 'vitest';

describe('GeneralApi', () => {
    it('getCity should return a promise', () => {
        const result = generalApi.getCity();
        expect(result).toBeInstanceOf(Promise);
    });

    it('getStation should return a promise', () => {
        const result = generalApi.getStation();
        expect(result).toBeInstanceOf(Promise);
    });
});