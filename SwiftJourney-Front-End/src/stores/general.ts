import { defineStore } from 'pinia'
import { generalApi } from '@/api/GeneralApi/generalApi';
import { pinyin } from 'pinyin-pro'

export const useGeneralStore = defineStore('general', {
    state: () => ({
        hasInit: false,
        CityMapPinYin: {} as { [key: string]: string[] },
        PinYinMapCity: {} as { [key: string]: string[] },
        PinYinList: [] as string[],
        CityMap: {} as { [province: string]: string[] }, // 后端原始数据
        CityPinYinList: [] as {cityName: string, pinYin: string}[],
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
                        const cityPinYinListBatch: {cityName: string, pinYin: string}[] = [];

                        for (const province in cityMap) {
                            const cities = cityMap[province];
                            cities.forEach(city => {
                                const pinYin = pinyin(city, { toneType: 'none' });

                                cityPinYinListBatch.push({cityName: city, pinYin: pinYin});

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

                        for (const pinYin in pinYinMapCityBatch) {
                            this.PinYinList.push(pinYin);
                        }

                        this.PinYinMapCity = pinYinMapCityBatch;
                        this.CityMapPinYin = cityMapPinYinBatch;
                        this.CityPinYinList = cityPinYinListBatch;

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