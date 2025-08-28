import { useGeneralStore } from '../../src/stores/general';
import { describe, it, expect } from 'vitest';

const generalStore = useGeneralStore();

describe('generalStore', () => {
    it('status of init function should be successful', async () => {
        await generalStore.init();
        expect(generalStore.hasInit).toBe(true);
    });
    it('handleData function should sussessfully process data', () => {
        const handleResult:{
            cityMapPinYinBatch: { [key: string]: string[] },
            pinYinMapCityBatch: { [key: string]: string[] },
            cityPinYinListBatch: { cityName: string, pinYin: string }[],
            pinYinList: string[],
            set: Set<string>,
        } = generalStore.handleData(generalStore.CityMap, generalStore.BothPinYinList);
        expect(handleResult.cityMapPinYinBatch.length).toBeGreaterThan(0);
        expect(handleResult.pinYinMapCityBatch.length).toBeGreaterThan(0);
        expect(handleResult.cityPinYinListBatch.length).toBeGreaterThan(0);
        expect(handleResult.pinYinList.length).toBeGreaterThan(0);
        expect(handleResult.set.size).toBeGreaterThan(0);
    });
    it('checkInputString function should correctly identify city and station', () => {
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