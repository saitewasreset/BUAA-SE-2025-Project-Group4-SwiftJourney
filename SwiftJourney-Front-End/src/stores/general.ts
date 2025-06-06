import { defineStore } from 'pinia'
import { generalApi } from '@/api/GeneralApi/generalApi';
import { pinyin } from 'pinyin-pro'

export const useGeneralStore = defineStore('general', {
    state: () => ({
        hasInit: false,
        CityMapPinYin: new Map<string, string[]>(),
        PinYinMapCity: new Map<string, string[]>(),
        PinYinList: [] as string[],
        CityMap: {}, //后端原始数据
    }),
    actions: {
        async init() {
            if(this.hasInit){
                return;
            }
            await generalApi.getCity().then((res) => {
                if(res.status == 200) {
                    if(res.data.code == 200) {
                        let cityMap: { [province: string]: string[] } = res.data.data;
                        this.CityMap = cityMap;
                        for (const province in cityMap) {
                            const cities = cityMap[province];
                            cities.forEach(city => this.addCityToMap(city));
                        }
                        this.PinYinMapCity.forEach((vlaue, key) => {
                            this.PinYinList.push(key);
                        })
                        this.hasInit = true;
                    } else {
                        throw new Error(res.data.message);
                    }
                }
            }).catch((err) => {
                console.log(err);
            })
        },
        addCityToMap(city: string) {
            let pinYin = pinyin(city, { toneType: 'none' });
            for (let i = 1; i < city.length; i++) {
                const key = city.substring(0, i);
                if (!this.CityMapPinYin.has(key)) {
                    this.CityMapPinYin.set(key, []);
                }
                let cities = this.CityMapPinYin.get(key);
                if (cities != null) {
                    cities.push(pinYin);
                    this.CityMapPinYin.set(key, cities);
                }
            }
            if(!this.PinYinMapCity.has(pinYin)) {
                this.PinYinMapCity.set(pinYin, []);
            }
            let pinyins = this.PinYinMapCity.get(pinYin);
            if(pinyins != null) {
                pinyins.push(city);
                this.PinYinMapCity.set(pinYin, pinyins);
            }
        }
    }
});