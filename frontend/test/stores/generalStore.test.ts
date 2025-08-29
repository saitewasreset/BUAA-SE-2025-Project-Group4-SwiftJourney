import { describe, it, expect, beforeEach, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { useGeneralStore } from '../../src/stores/general';

describe('generalStore', () => {
    let generalStore: ReturnType<typeof useGeneralStore>;

    beforeEach(() => {
        // 创建并激活 Pinia 实例
        const pinia = createPinia();
        setActivePinia(pinia);
        
        // 现在可以安全地创建 store
        generalStore = useGeneralStore();
        
        // Mock API 请求
        vi.doMock('../../src/api/GeneralApi/generalApi', () => ({
            generalApi: {
                getCity: vi.fn().mockResolvedValue({
                    data: {
                        cityMap: {
                            '北京': ['北京', '北京西', '北京南', '北京东'],
                            '上海': ['上海', '上海虹桥', '上海南'],
                            '广州': ['广州', '广州东', '广州南'],
                            '深圳': ['深圳', '深圳北', '深圳东']
                        },
                        bothPinYinList: [
                            { cityName: '北京', pinYin: 'beijing' },
                            { cityName: '上海', pinYin: 'shanghai' },
                            { cityName: '广州', pinYin: 'guangzhou' },
                            { cityName: '深圳', pinYin: 'shenzhen' }
                        ]
                    }
                })
            }
        }));
    });

    it('status of init function should be successful', async () => {
        await generalStore.init();
        expect(generalStore.hasInit).toBe(true);
    });

    it('handleData function should successfully process data', async () => {
        // 先确保 store 已初始化
        await generalStore.init();
        
        const handleResult: {
            cityMapPinYinBatch: { [key: string]: string[] },
            pinYinMapCityBatch: { [key: string]: string[] },
            cityPinYinListBatch: { cityName: string, pinYin: string }[],
            pinYinList: string[],
            set: Set<string>,
        } = generalStore.handleData(generalStore.CityMap, generalStore.BothPinYinList);
        
        // 修正断言 - 对象没有 length 属性，使用 Object.keys().length
        expect(Object.keys(handleResult.cityMapPinYinBatch).length).toBeGreaterThan(0);
        expect(Object.keys(handleResult.pinYinMapCityBatch).length).toBeGreaterThan(0);
        expect(handleResult.cityPinYinListBatch.length).toBeGreaterThan(0);
        expect(handleResult.pinYinList.length).toBeGreaterThan(0);
        expect(handleResult.set.size).toBeGreaterThan(0);
    });

    it('checkInputString function should correctly identify city and station', async () => {
        // 先确保 store 已初始化
        await generalStore.init();
        
        let checkResult: { targetType: 'city' | 'station', target: string } | undefined;
        
        checkResult = generalStore.checkInputString('北京西站');
        expect(checkResult).toEqual({ targetType: 'station', target: '北京西' });
        
        checkResult = generalStore.checkInputString('高松灯站');
        expect(checkResult).toEqual(undefined);
        
        checkResult = generalStore.checkInputString('上海');
        expect(checkResult).toEqual({ targetType: 'city', target: '上海' });
        
        checkResult = generalStore.checkInputString('不存在的城市');
        expect(checkResult).toEqual(undefined);
    });
});