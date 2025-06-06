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
        CitySet: new Set<string>(),
        
        StationMapPinYin: {} as { [key: string]: string[] },
        PinYinMapStation: {} as { [key: string]: string[] },
        PinYinListStation: [] as string[],
        StationMap: {} as { [city: string]: string[] }, // 后端原始数据
        NewStationMap: {} as { [city: string]: string[] }, // + '站' 数据
        StationPinYinList: [] as {cityName: string, pinYin: string}[],
        StationSet: new Set<string>(),

        BothPinYinList: [] as {cityName: string, pinYin: string}[],
    }),
    actions: {
        async init() {
            if (this.hasInit) {
                return;
            }

            let successGetCity = false;
            let successGetStation = false;

            let bothPinYinList: {cityName: string, pinYin: string}[] = [];

            try {
                const res = await generalApi.getCity();
                if (res.status === 200) {
                    if (res.data.code === 200) {
                        const cityMap: { [province: string]: string[] } = res.data.data;
                        this.CityMap = cityMap;

                        const result = this.handleData(cityMap, bothPinYinList);

                        this.PinYinMapCity = result.pinYinMapCityBatch;
                        this.CityMapPinYin = result.cityMapPinYinBatch;
                        this.CityPinYinList = result.cityPinYinListBatch;
                        this.PinYinList = result.pinYinList;
                        this.CitySet = result.set;

                        successGetCity = true;
                    } else {
                        throw new Error(res.data.message);
                    }
                }

                const ress = await generalApi.getStation();
                if(ress.status == 200) {
                    if(ress.data.code == 200) {
                        const stationMap: { [city: string]: string[] } = ress.data.data;
                        this.StationMap = stationMap;

                        const newStationMap: { [city: string]: string[] } = {};

                        for(const city in stationMap) {
                            const stations = stationMap[city];
                            const newStations: string[] = [];
                            stations.forEach((value) =>{
                                newStations.push(value + '站');
                            })
                            newStationMap[city] = newStations;
                        }
                        this.NewStationMap = newStationMap;

                        const result = this.handleData(newStationMap, bothPinYinList);

                        this.PinYinMapStation = result.pinYinMapCityBatch;
                        this.StationMapPinYin = result.cityMapPinYinBatch;
                        this.StationPinYinList = result.cityPinYinListBatch;
                        this.PinYinListStation = result.pinYinList;
                        this.StationSet = result.set;

                    } else {
                        throw new Error(ress.data.message);
                    }
                }
            } catch (err) {
                console.log(err);
            }

            this.BothPinYinList = bothPinYinList;
            this.hasInit = successGetCity && successGetStation;
        },
        handleData (cityMap: { [province: string]: string[] }, bothPinYinList: {cityName: string, pinYin: string}[]) {
            // 准备批量更新的数据
            const cityMapPinYinBatch: { [key: string]: string[] } = {};
            const pinYinMapCityBatch: { [key: string]: string[] } = {};
            const cityPinYinListBatch: {cityName: string, pinYin: string}[] = [];
            const pinYinList: string[] = [];
            const set = new Set<string>();

            for (const province in cityMap) {
                const cities = cityMap[province];
                cities.forEach(city => {
                    set.add(city);

                    const pinYin = pinyin(city, { toneType: 'none' });

                    cityPinYinListBatch.push({cityName: city, pinYin: pinYin});
                    bothPinYinList.push({cityName: city, pinYin: pinYin});

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
                pinYinList.push(pinYin);
            }

            return { 
                cityMapPinYinBatch: cityMapPinYinBatch, 
                pinYinMapCityBatch: pinYinMapCityBatch,
                cityPinYinListBatch: cityPinYinListBatch,
                pinYinList: pinYinList,
                set: set,
            };
        },
        checkInputString(input: string): {targetType: 'city' | 'station', target: string} | undefined {
            if(input.endsWith('站')) {
                if(this.StationSet.has(input)) {
                    return { targetType: 'station', target: input.slice(0, -1) };
                } else {
                    return undefined;
                }
            }
            if(this.CitySet.has(input)) {
                return { targetType: 'city', target: input };
            } else if (this.StationSet.has(input + '站')) {
                return { targetType: 'station', target: input };
            } else {
                return undefined;
            }
        }
    }
});