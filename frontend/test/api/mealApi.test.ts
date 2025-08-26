import { mealApi } from '../../src/api/MealApi/mealApi';
import { describe, it, expect } from 'vitest';


describe('mealApi', () => {
    it('dishQuery should return a promise', () => {
        const result = mealApi.dishQuery({});
        expect(result).toBeInstanceOf(Promise);
    });

    it('dishOrder should return a promise', () => {
        const result = mealApi.dishOrder({});
        expect(result).toBeInstanceOf(Promise);
    });
});
