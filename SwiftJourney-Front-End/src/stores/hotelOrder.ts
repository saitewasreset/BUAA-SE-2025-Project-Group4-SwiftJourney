import { defineStore } from "pinia";
import type { HotelOrderInfo, HotelRoomInfo, HotelDetailInfo } from "@/interface/hotelInterface";
import { useUserStore } from "./user";
const userStore = useUserStore();

export const useHotelOrderStore = defineStore('hotelOrder', {
    state: () => ({
        hotelOrderInfoList: [] as HotelOrderInfo[],
    }),
    actions: {
        add(room: HotelRoomInfo, hotel: HotelDetailInfo | undefined, beginDate: string, endDate: string): boolean {
            if(hotel == undefined) {
                return false;
            }
            let hotelOrderInfo: HotelOrderInfo = {
                name: hotel.name,
                hotelId: hotel.hotelId,
                maxCount: room.remainCount,
                roomType: room.roomType,
                amount: 1,
                price: room.price,
                personalId: userStore.personalId,
                beginDate: beginDate,
                endDate: endDate,
            }
            for (let key of this.hotelOrderInfoList) {
                if (hotel.hotelId === key.hotelId && room.roomType === key.roomType && beginDate == key.beginDate && endDate == key.endDate) {
                    return false;
                }
            }
            this.hotelOrderInfoList.push(hotelOrderInfo);
            this.syncToLocalStorage();
            return true;
        },
        delete(hotelId: string, roomType: string, beginDate: string, endDate: string) {
            this.hotelOrderInfoList = this.hotelOrderInfoList.filter(key => !(hotelId == key.hotelId && roomType == key.roomType && beginDate == key.beginDate && endDate == key.endDate));
            this.syncToLocalStorage();
        },
        syncToLocalStorage() {
            localStorage.setItem('hotelOrderInfoList', JSON.stringify(this.hotelOrderInfoList));
        },
        loadFromLocalStorage() {
            const storedList = localStorage.getItem('hotelOrderInfoList');
            if (storedList) {
                this.hotelOrderInfoList = JSON.parse(storedList) as HotelOrderInfo[];
            }
        },
        deleteAll() {
            this.hotelOrderInfoList = [];
            this.syncToLocalStorage();
        }
    },
})