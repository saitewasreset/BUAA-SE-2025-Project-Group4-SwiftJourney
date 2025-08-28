import { setActivePinia, createPinia } from "pinia";
import { describe, it, beforeEach, vi, expect } from "vitest";
import { useHotelOrderStore } from "../../src/stores/hotelOrder";

// Mock user store and localStorage
vi.mock("@/stores/user", () => ({
    useUserStore: () => ({
        personalId: "test-personal-id"
    })
}));

// Define complete mock interfaces that match the actual Room and Hotel types
interface MockRoom {
    remainCount: number;
    roomType: string;
    price: number;
    // Add any other properties that the actual Room interface might have
    id?: string;
    description?: string;
    amenities?: string[];
}

interface MockHotel {
    name: string;
    hotelId: string;
    // Add any other properties that the actual Hotel interface might have
    address?: string;
    rating?: number;
    description?: string;
}

const mockRoom: MockRoom = {
    remainCount: 5,
    roomType: "Deluxe",
    price: 200,
    id: "room-001",
    description: "Luxury deluxe room",
    amenities: ["WiFi", "TV", "Air Conditioning"]
};

const mockHotel: MockHotel = {
    name: "Test Hotel",
    hotelId: "hotel-123",
    address: "123 Test Street",
    rating: 4.5,
    description: "A wonderful test hotel"
};

const beginDate = "2024-06-01";
const endDate = "2024-06-05";

describe("hotelOrder store", () => {
    let store: ReturnType<typeof useHotelOrderStore>;

    beforeEach(() => {
        setActivePinia(createPinia());
        store = useHotelOrderStore();
        
        // Mock localStorage
        const localStorageMock = (() => {
            let store: Record<string, string> = {};
            return {
                getItem: (key: string) => store[key] || null,
                setItem: (key: string, value: string) => { store[key] = value; },
                removeItem: (key: string) => { delete store[key]; },
                clear: () => { store = {}; }
            };
        })();
        
        Object.defineProperty(window, "localStorage", {
            value: localStorageMock,
            writable: true
        });
        
        // Clear store before each test
        store.deleteAll();
    });

    it("adds a hotel order successfully", () => {
        const result = store.add(mockRoom, mockHotel, beginDate, endDate);
        expect(result).toBe(true);
        expect(store.hotelOrderInfoList.length).toBe(1);
        expect(store.hotelOrderInfoList[0].hotelId).toBe("hotel-123");
        expect(store.hotelOrderInfoList[0].roomType).toBe("Deluxe");
        expect(store.hotelOrderInfoList[0].personalId).toBe("test-personal-id");
    });

    it("does not add duplicate hotel orders", () => {
        store.add(mockRoom, mockHotel, beginDate, endDate);
        const result = store.add(mockRoom, mockHotel, beginDate, endDate);
        expect(result).toBe(false);
        expect(store.hotelOrderInfoList.length).toBe(1);
    });

    it("does not add if hotel is undefined", () => {
        const result = store.add(mockRoom, undefined, beginDate, endDate);
        expect(result).toBe(false);
        expect(store.hotelOrderInfoList.length).toBe(0);
    });

    it("deletes a hotel order", () => {
        store.add(mockRoom, mockHotel, beginDate, endDate);
        expect(store.hotelOrderInfoList.length).toBe(1);
        
        store.delete("hotel-123", "Deluxe", beginDate, endDate);
        expect(store.hotelOrderInfoList.length).toBe(0);
    });

    it("syncs and loads from localStorage", () => {
        store.add(mockRoom, mockHotel, beginDate, endDate);
        expect(JSON.parse(window.localStorage.getItem("hotelOrderInfoList")!)).toHaveLength(1);

        // Clear store and reload from localStorage
        store.hotelOrderInfoList = [];
        store.loadFromLocalStorage();
        expect(store.hotelOrderInfoList).toHaveLength(1);
        expect(store.hotelOrderInfoList[0].hotelId).toBe("hotel-123");
    });

    it("deletes all hotel orders", () => {
        store.add(mockRoom, mockHotel, beginDate, endDate);
        const mockSuiteRoom: MockRoom = { 
            ...mockRoom, 
            roomType: "Suite",
            id: "room-002",
            description: "Luxury suite room"
        };
        store.add(mockSuiteRoom, mockHotel, beginDate, endDate);
        expect(store.hotelOrderInfoList.length).toBe(2);
        
        store.deleteAll();
        expect(store.hotelOrderInfoList.length).toBe(0);
        expect(JSON.parse(window.localStorage.getItem("hotelOrderInfoList")!)).toHaveLength(0);
    });
});