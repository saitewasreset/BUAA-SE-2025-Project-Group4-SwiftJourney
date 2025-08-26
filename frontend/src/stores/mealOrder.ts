import { defineStore } from "pinia";
import { useUserStore } from "./user";
import type { MealOrder, TakeawayDishInfo } from "@/interface/mealInterface";
const userStore = useUserStore();

export const useMealOrderStore = defineStore('mealOrder', {
    state: () => ({
        mealOrderInfoList: [] as MealOrder[],
        trainNumber: '',
        originDepartureTime: '',
    }),
    actions: {
        add(trainNumber: string, originDepartureTime: string, shopName: string, 
            food: TakeawayDishInfo, station?: string, dishTime?: 'lunch' | 'dinner'): boolean {

            if(this.mealOrderInfoList.length == 0) {
                this.trainNumber = trainNumber;
                this.originDepartureTime = originDepartureTime;
            } else if(trainNumber != this.trainNumber || originDepartureTime != this.originDepartureTime) {
                return false;
            }

            const mealOrderInfo: MealOrder = {
                shopName: shopName,
                name: food.name,
                amount: 1,
                price: food.price,
                personalId: userStore.personalId,
                station: station,
                dishTime: dishTime,
            }
            for (let key of this.mealOrderInfoList) {
                if (key.shopName == shopName && key.name == food.name && key.dishTime == dishTime) {
                    key.amount ++;
                    return true;
                }
            }
            this.mealOrderInfoList.push(mealOrderInfo);
            return true;
        },
        delete(shopName: string, foodName: string, dishTime?: 'lunch' | 'dinner') {
            this.mealOrderInfoList = this.mealOrderInfoList.filter(key => 
                !(shopName == key.shopName && foodName == key.name && dishTime == key.dishTime)
            );
        },
        deleteAll() {
            this.mealOrderInfoList = [];
        },
    }
});