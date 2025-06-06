import { defineStore } from 'pinia'
import { generalApi } from '@/api/GeneralApi/generalApi';
import { pinyin } from 'pinyin-pro'

export const useGeneralStore = defineStore('general', {
    state: () => ({
        hasInit: false,
        CityMapPinYin: new Map<string, string[]>(),
        PinYinMapCity: new Map<string, string[]>(),
        PinYinList: [] as string[],
        CityMap: {}, // 后端原始数据
    }),
    actions: {
        async init() {
            if (this.hasInit) {
                return;
            }
            try {
                const res = await generalApi.getCity();
                if (res.status === 200) {
                    if (res.data.code === 200) {
                        const cityMap: { [province: string]: string[] } = res.data.data;
                        this.CityMap = cityMap;

                        // 准备批量更新的数据
                        const cityMapPinYinBatch: { [key: string]: string[] } = {};
                        const pinYinMapCityBatch: { [key: string]: string[] } = {};

                        for (const province in cityMap) {
                            const cities = cityMap[province];
                            cities.forEach(city => {
                                const pinYin = pinyin(city, { toneType: 'none' });

                                // 更新CityMapPinYinBatch
                                for (let i = 1; i <= city.length; i++) {
                                    const key = city.substring(0, i);
                                    if (!cityMapPinYinBatch[key]) {
                                        cityMapPinYinBatch[key] = [];
                                    }
                                    cityMapPinYinBatch[key].push(pinYin);
                                }

                                // 更新pinYinMapCityBatch
                                if (!pinYinMapCityBatch[pinYin]) {
                                    pinYinMapCityBatch[pinYin] = [];
                                }
                                pinYinMapCityBatch[pinYin].push(city);
                            });
                        }

                        // 批量更新CityMapPinYin
                        for (const key in cityMapPinYinBatch) {
                            this.CityMapPinYin.set(key, cityMapPinYinBatch[key]);
                        }

                        // 批量更新PinYinMapCity
                        for (const pinYin in pinYinMapCityBatch) {
                            this.PinYinMapCity.set(pinYin, pinYinMapCityBatch[pinYin]);
                            this.PinYinList.push(pinYin);
                        }

                        this.hasInit = true;
                    } else {
                        throw new Error(res.data.message);
                    }
                }
            } catch (err) {
                console.log(err);
            }
        },
    }
});