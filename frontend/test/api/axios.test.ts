import { getRequest, postRequest, postBlobRequest, putRequest } from "../../src/api/axios";
import { describe, it, expect } from "vitest";

describe('axios', () => {
    it('getRequest should return a promise and resolve successfully', async () => {
        const result = getRequest('/test', { param1: 'value1' });
        expect(result).toBeInstanceOf(Promise);
        
        const response = await result;
        expect(response.data).toEqual({ msg: "ok" });
    });

    it('postRequest should return a promise and resolve successfully', async () => {
        const result = postRequest('/test', { key1: 'value1' });
        expect(result).toBeInstanceOf(Promise);
        
        const response = await result;
        expect(response.data).toEqual({ msg: "posted" });
    });

    it('postBlobRequest should return a promise and resolve successfully', async () => {
        const result = postBlobRequest('/test/blob', { key1: 'value1' });
        expect(result).toBeInstanceOf(Promise);
        
        const response = await result;
        expect(response.data).toEqual(new Blob());
    });

    it('putRequest should return a promise and resolve successfully', async () => {
        const result = putRequest('/test', { key1: 'value1' });
        expect(result).toBeInstanceOf(Promise);
        
        const response = await result;
        expect(response.data).toEqual({ msg: "updated" });
    });

    it('should handle API errors gracefully', async () => {
        // 测试不存在的端点
        const result = getRequest('/nonexistent');
        await expect(result).rejects.toThrow();
    });
});