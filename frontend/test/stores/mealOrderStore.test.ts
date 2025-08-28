import { setActivePinia, createPinia } from "pinia";
import { describe, it, beforeEach, vi, expect } from "vitest";
import { useMealOrderStore } from "../../src/stores/mealOrder";

// Mock user store
vi.mock("@/stores/user", () => ({
    useUserStore: () => ({
        personalId: "test-personal-id"
    })
}));

// Define mock interfaces
interface MockTakeawayDishInfo {
    name: string;
    price: number;
    description?: string;
    category?: string;
    available?: boolean;
}

const mockFood: MockTakeawayDishInfo = {
    name: "Beef Noodles",
    price: 25.5,
    description: "Delicious beef noodles",
    category: "noodles",
    available: true
};

const mockFood2: MockTakeawayDishInfo = {
    name: "Chicken Rice",
    price: 18.0,
    description: "Tasty chicken rice",
    category: "rice",
    available: true
};

describe("mealOrder store", () => {
    let store: ReturnType<typeof useMealOrderStore>;

    beforeEach(() => {
        setActivePinia(createPinia());
        store = useMealOrderStore();
        store.deleteAll();
    });

    describe("add method", () => {
        it("adds first meal order successfully", () => {
            const result = store.add(
                "G1234", 
                "2024-06-01 08:00:00", 
                "Test Restaurant", 
                mockFood, 
                "Shanghai", 
                "lunch"
            );

            expect(result).toBe(true);
            expect(store.mealOrderInfoList.length).toBe(1);
            expect(store.trainNumber).toBe("G1234");
            expect(store.originDepartureTime).toBe("2024-06-01 08:00:00");
            expect(store.mealOrderInfoList[0].shopName).toBe("Test Restaurant");
            expect(store.mealOrderInfoList[0].name).toBe("Beef Noodles");
            expect(store.mealOrderInfoList[0].amount).toBe(1);
            expect(store.mealOrderInfoList[0].price).toBe(25.5);
            expect(store.mealOrderInfoList[0].personalId).toBe("test-personal-id");
            expect(store.mealOrderInfoList[0].station).toBe("Shanghai");
            expect(store.mealOrderInfoList[0].dishTime).toBe("lunch");
        });

        it("adds meal order with same train and departure time", () => {
            store.add("G1234", "2024-06-01 08:00:00", "Test Restaurant", mockFood, "Shanghai", "lunch");
            const result = store.add("G1234", "2024-06-01 08:00:00", "Another Restaurant", mockFood2, "Beijing", "dinner");

            expect(result).toBe(true);
            expect(store.mealOrderInfoList.length).toBe(2);
            expect(store.mealOrderInfoList[1].shopName).toBe("Another Restaurant");
            expect(store.mealOrderInfoList[1].name).toBe("Chicken Rice");
        });

        it("rejects meal order with different train number", () => {
            store.add("G1234", "2024-06-01 08:00:00", "Test Restaurant", mockFood);
            const result = store.add("G5678", "2024-06-01 08:00:00", "Another Restaurant", mockFood2);

            expect(result).toBe(false);
            expect(store.mealOrderInfoList.length).toBe(1);
            expect(store.trainNumber).toBe("G1234");
        });

        it("rejects meal order with different departure time", () => {
            store.add("G1234", "2024-06-01 08:00:00", "Test Restaurant", mockFood);
            const result = store.add("G1234", "2024-06-01 10:00:00", "Another Restaurant", mockFood2);

            expect(result).toBe(false);
            expect(store.mealOrderInfoList.length).toBe(1);
            expect(store.originDepartureTime).toBe("2024-06-01 08:00:00");
        });

        it("increments amount for duplicate meal order", () => {
            store.add("G1234", "2024-06-01 08:00:00", "Test Restaurant", mockFood, "Shanghai", "lunch");
            const result = store.add("G1234", "2024-06-01 08:00:00", "Test Restaurant", mockFood, "Shanghai", "lunch");

            expect(result).toBe(true);
            expect(store.mealOrderInfoList.length).toBe(1);
            expect(store.mealOrderInfoList[0].amount).toBe(2);
        });

        it("adds separate orders for same food but different dish time", () => {
            store.add("G1234", "2024-06-01 08:00:00", "Test Restaurant", mockFood, "Shanghai", "lunch");
            const result = store.add("G1234", "2024-06-01 08:00:00", "Test Restaurant", mockFood, "Shanghai", "dinner");

            expect(result).toBe(true);
            expect(store.mealOrderInfoList.length).toBe(2);
            expect(store.mealOrderInfoList[0].dishTime).toBe("lunch");
            expect(store.mealOrderInfoList[1].dishTime).toBe("dinner");
        });

        it("adds meal order without optional parameters", () => {
            const result = store.add("G1234", "2024-06-01 08:00:00", "Test Restaurant", mockFood);

            expect(result).toBe(true);
            expect(store.mealOrderInfoList.length).toBe(1);
            expect(store.mealOrderInfoList[0].station).toBeUndefined();
            expect(store.mealOrderInfoList[0].dishTime).toBeUndefined();
        });
    });

    describe("delete method", () => {
        beforeEach(() => {
            store.add("G1234", "2024-06-01 08:00:00", "Test Restaurant", mockFood, "Shanghai", "lunch");
            store.add("G1234", "2024-06-01 08:00:00", "Test Restaurant", mockFood2, "Beijing", "dinner");
            store.add("G1234", "2024-06-01 08:00:00", "Another Restaurant", mockFood, "Shanghai", "lunch");
        });

        it("deletes specific meal order", () => {
            expect(store.mealOrderInfoList.length).toBe(3);
            
            store.delete("Test Restaurant", "Beef Noodles", "lunch");
            
            expect(store.mealOrderInfoList.length).toBe(2);
            expect(store.mealOrderInfoList.find(order => 
                order.shopName === "Test Restaurant" && 
                order.name === "Beef Noodles" && 
                order.dishTime === "lunch"
            )).toBeUndefined();
        });

        it("deletes meal order without dish time", () => {
            store.add("G1234", "2024-06-01 08:00:00", "Test Restaurant", mockFood);
            expect(store.mealOrderInfoList.length).toBe(4);
            
            store.delete("Test Restaurant", "Beef Noodles");
            
            expect(store.mealOrderInfoList.length).toBe(3);
        });

        it("does not delete non-matching orders", () => {
            store.delete("Nonexistent Restaurant", "Beef Noodles", "lunch");
            
            expect(store.mealOrderInfoList.length).toBe(3);
        });

        it("only deletes exact matches", () => {
            store.delete("Test Restaurant", "Beef Noodles", "dinner");
            
            expect(store.mealOrderInfoList.length).toBe(3);
            expect(store.mealOrderInfoList.find(order => 
                order.shopName === "Test Restaurant" && 
                order.name === "Beef Noodles" && 
                order.dishTime === "lunch"
            )).toBeDefined();
        });
    });

    describe("deleteAll method", () => {
        it("clears all meal orders", () => {
            store.add("G1234", "2024-06-01 08:00:00", "Test Restaurant", mockFood, "Shanghai", "lunch");
            store.add("G1234", "2024-06-01 08:00:00", "Another Restaurant", mockFood2, "Beijing", "dinner");
            
            expect(store.mealOrderInfoList.length).toBe(2);
            
            store.deleteAll();
            
            expect(store.mealOrderInfoList.length).toBe(0);
        });

        it("can add new orders after clearing", () => {
            store.add("G1234", "2024-06-01 08:00:00", "Test Restaurant", mockFood);
            store.deleteAll();
            
            const result = store.add("G5678", "2024-06-02 10:00:00", "New Restaurant", mockFood2);
            
            expect(result).toBe(true);
            expect(store.mealOrderInfoList.length).toBe(1);
            expect(store.trainNumber).toBe("G5678");
            expect(store.originDepartureTime).toBe("2024-06-02 10:00:00");
        });
    });

    describe("state management", () => {
        it("maintains train number and departure time across orders", () => {
            store.add("G1234", "2024-06-01 08:00:00", "Restaurant A", mockFood);
            store.add("G1234", "2024-06-01 08:00:00", "Restaurant B", mockFood2);
            
            expect(store.trainNumber).toBe("G1234");
            expect(store.originDepartureTime).toBe("2024-06-01 08:00:00");
        });

        it("resets train info when starting fresh", () => {
            store.add("G1234", "2024-06-01 08:00:00", "Restaurant A", mockFood);
            store.deleteAll();
            store.add("G5678", "2024-06-02 10:00:00", "Restaurant B", mockFood2);
            
            expect(store.trainNumber).toBe("G5678");
            expect(store.originDepartureTime).toBe("2024-06-02 10:00:00");
        });
    });

    describe("edge cases", () => {
        it("handles multiple increments of same order", () => {
            const shopName = "Test Restaurant";
            const dishTime = "lunch";
            
            store.add("G1234", "2024-06-01 08:00:00", shopName, mockFood, "Shanghai", dishTime);
            store.add("G1234", "2024-06-01 08:00:00", shopName, mockFood, "Shanghai", dishTime);
            store.add("G1234", "2024-06-01 08:00:00", shopName, mockFood, "Shanghai", dishTime);
            
            expect(store.mealOrderInfoList.length).toBe(1);
            expect(store.mealOrderInfoList[0].amount).toBe(3);
        });

        it("handles orders with different stations for same food", () => {
            store.add("G1234", "2024-06-01 08:00:00", "Test Restaurant", mockFood, "Shanghai", "lunch");
            store.add("G1234", "2024-06-01 08:00:00", "Test Restaurant", mockFood, "Beijing", "lunch");
            
            expect(store.mealOrderInfoList.length).toBe(2);
            expect(store.mealOrderInfoList[0].station).toBe("Shanghai");
            expect(store.mealOrderInfoList[1].station).toBe("Beijing");
        });
    });
});