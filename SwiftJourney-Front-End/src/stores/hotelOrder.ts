import { defineStore } from "pinia";
import type { HotelOrderInfo, HotelRoomInfo, HotelDetailInfo } from "@/interface/hotelInterface";
import { useUserStore } from "./user";
const userStore = useUserStore();

export const useHotelOrderStore = defineStore('hotelOrder', {
    state: () => ({
        hotelOrderInfoList: [] as HotelOrderInfo[],
    }),
    actions: {
        add(room: HotelRoomInfo, hotel: HotelDetailInfo | undefined): boolean {
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
                personalId: userStore.phone,
            }
            for (let key of this.hotelOrderInfoList) {
                if (hotel.hotelId === key.hotelId && room.roomType === key.roomType) {
                    return false;
                }
            }
            this.hotelOrderInfoList.push(hotelOrderInfo);
            return true;
        },
        delete(hotelId: string, roomType: string) {
            this.hotelOrderInfoList = this.hotelOrderInfoList.filter(key => !(hotelId == key.hotelId && roomType == key.roomType));
        },
    },
})