import { getRequest, postRequest, postBlobRequest, putRequest } from "../../src/api/axios";
import { describe, it, expect } from "vitest";

describe('axios', () => {
    it('getRequest should return a promise', () => {
        const result = getRequest('/test', { param1: 'value1' });
        expect(result).toBeInstanceOf(Promise);
    });

    it('postRequest should return a promise', () => {
        const result = postRequest('/test', { key1: 'value1' });
        expect(result).toBeInstanceOf(Promise);
    });

    it('postBlobRequest should return a promise', () => {
        const result = postBlobRequest('/test', { key1: 'value1' });
        expect(result).toBeInstanceOf(Promise);
    });

    it('putRequest should return a promise', () => {
        const result = putRequest('/test', { key1: 'value1' });
        expect(result).toBeInstanceOf(Promise);
    });
});