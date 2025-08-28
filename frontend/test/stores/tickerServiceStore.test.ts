import { setActivePinia, createPinia } from "pinia";
import { describe, it, beforeEach, vi, expect } from "vitest";
import { useTicketServiceStore } from "../../src/stores/ticketService";
import dayjs, { Dayjs } from "dayjs";
import { CheckType, SortType } from '../../src/interface/ticketServiceInterface'
import customParseFormat from "dayjs/plugin/customParseFormat";
dayjs.extend(customParseFormat);

// Mock dependencies
vi.mock("@/stores/general", () => ({
    useGeneralStore: () => ({
        checkInputString: vi.fn((input: string) => {
            if (input === "北京") return { targetType: "city", target: "北京" };
            if (input === "上海") return { targetType: "city", target: "上海" };
            if (input === "北京南站") return { targetType: "station", target: "北京南站" };
            return undefined;
        })
    })
}));

vi.mock("@/api/train", () => ({
    queryDirectSchedule: vi.fn(),
    queryIndirectSchedule: vi.fn()
}));

vi.mock("ant-design-vue", () => ({
    message: {
        error: vi.fn(),
        success: vi.fn()
    }
}));

const mockDirectScheduleData = {
    code: 200,
    data: {
        solutions: [
            {
                trainNumber: "G1",
                departureStation: "北京南站",
                arrivalStation: "上海虹桥站",
                departureTime: "2025-09-01 08:00",
                arrivalTime: "2025-09-01 12:30",
                travelTime: "04:30",
                seatInfo: {
                    "二等座": { seatType: "二等座", left: 10, price: 553 },
                    "一等座": { seatType: "一等座", left: 5, price: 933 },
                    "商务座": { seatType: "商务座", left: 2, price: 1748 }
                }
            },
            {
                trainNumber: "G3",
                departureStation: "北京南站",
                arrivalStation: "上海虹桥站",
                departureTime: "2025-09-01 09:00",
                arrivalTime: "2025-09-01 13:28",
                travelTime: "04:28",
                seatInfo: {
                    "二等座": { seatType: "二等座", left: 0, price: 553 },
                    "一等座": { seatType: "一等座", left: 0, price: 933 },
                    "商务座": { seatType: "商务座", left: 0, price: 1748 }
                }
            }
        ]
    }
};


describe("ticketService store", () => {
    let store: ReturnType<typeof useTicketServiceStore>;

    beforeEach(() => {
        setActivePinia(createPinia());
        store = useTicketServiceStore();
        vi.clearAllMocks();
    });

    describe("initial state", () => {
        it("has correct default values", () => {
            expect(store.onlyShowAvailable).toBe(false);
            expect(store.isLoading).toBe(false);
            expect(store.sortType).toBe(SortType.DepartureTime);
            expect(store.sortOrderAsc).toBe(true);
            expect(store.queryMode).toBe("direct");
            expect(store.queryResult).toEqual([]);
            expect(store.preOrderSchedule).toBeNull();
        });

        it("has correct default query date", () => {
            const today = new Date();
            const expectedDate = `${today.getFullYear()}-${(today.getMonth() + 1).toString().padStart(2, '0')}-${today.getDate().toString().padStart(2, '0')}`;
            expect(store.queryDate).toBe(expectedDate);
        });

        it("has correct default time ranges", () => {
            expect(store.startTimeRangeNumber).toEqual([0, 1439]);
            expect(store.endTimeRangeNumber).toEqual([0, 1439]);
            expect(store.startTimeRange).toEqual([dayjs('00:00', 'HH:mm'), dayjs('23:59', 'HH:mm')]);
            expect(store.endTimeRange).toEqual([dayjs('00:00', 'HH:mm'), dayjs('23:59', 'HH:mm')]);
        });

        it("has correct default checkbox groups", () => {
            expect(store.checkGroups).toHaveLength(5);
            expect(store.checkGroups[CheckType.TrainType].options).toEqual(['G/C', 'D', 'T', 'K', 'Z', '其他']);
            expect(store.checkGroups[CheckType.TrainType].checkAll).toBe(true);
            expect(store.checkGroups[CheckType.SeatType].options).toEqual(['加载中...']);
        });
    });

    describe("getters", () => {
        describe("dateRange", () => {
            it("generates 14 days range starting from today", () => {
                const dateRange = store.dateRange;
                expect(dateRange).toHaveLength(14);
                
                const today = new Date();
                const firstDate = dateRange[0] as {date: string, display: string};
                const expectedFirstDate = `${today.getFullYear()}-${(today.getMonth() + 1).toString().padStart(2, '0')}-${today.getDate().toString().padStart(2, '0')}`;
                expect(firstDate.date).toBe(expectedFirstDate);
            });

            it("has correct display format", () => {
                const dateRange = store.dateRange as {date: string, display: string}[];
                expect(dateRange[0].display).toMatch(/^\d{1,2}-\d{1,2}$/);
            });
        });

        describe("sort type getters", () => {
            it("correctly identifies sort by departure time", () => {
                store.sortType = SortType.DepartureTime;
                expect(store.isSortByDepartureTime).toBe(true);
                expect(store.isSortByTravelTime).toBe(false);
                expect(store.isSortByPrice).toBe(false);
            });

            it("correctly identifies sort by travel time", () => {
                store.sortType = SortType.TravelTime;
                expect(store.isSortByDepartureTime).toBe(false);
                expect(store.isSortByTravelTime).toBe(true);
                expect(store.isSortByPrice).toBe(false);
            });

            it("correctly identifies sort by price", () => {
                store.sortType = SortType.Price;
                expect(store.isSortByDepartureTime).toBe(false);
                expect(store.isSortByTravelTime).toBe(false);
                expect(store.isSortByPrice).toBe(true);
            });
        });

        describe("displaySchedules", () => {
            beforeEach(() => {
                store.queryResult = mockDirectScheduleData.data.solutions;
                store.queryMode = "direct";
                store.queryDate = "2025-09-01";
            });

            it("returns all schedules when no filters applied", () => {
                expect(store.displaySchedules).toHaveLength(2);
            });

            it("filters by available tickets only", () => {
                store.onlyShowAvailable = true;
                const filtered = store.displaySchedules;
                expect(filtered).toHaveLength(1);
                expect((filtered[0] as any).trainNumber).toBe("G1");
            });

            it("filters by train type", () => {
                store.checkGroups[CheckType.TrainType].checkedList = ["G/C"];
                const filtered = store.displaySchedules;
                expect(filtered).toHaveLength(2);
            });

            it("sorts by departure time ascending", () => {
                store.sortType = SortType.DepartureTime;
                store.sortOrderAsc = true;
                const sorted = store.displaySchedules;
                expect((sorted[0] as any).departureTime).toBe("2025-09-01 08:00");
                expect((sorted[1] as any).departureTime).toBe("2025-09-01 09:00");
            });

            it("sorts by departure time descending", () => {
                store.sortType = SortType.DepartureTime;
                store.sortOrderAsc = false;
                const sorted = store.displaySchedules;
                expect((sorted[0] as any).departureTime).toBe("2025-09-01 09:00");
                expect((sorted[1] as any).departureTime).toBe("2025-09-01 08:00");
            });
        });
    });

    describe("actions", () => {
        describe("onCheckAllBoxChange", () => {
            it("toggles check all state", () => {
                const initialState = store.checkGroups[CheckType.TrainType].checkAll;
                store.onCheckAllBoxChange(CheckType.TrainType);
                expect(store.checkGroups[CheckType.TrainType].checkAll).toBe(!initialState);
            });

            it("selects all options when checking all", () => {
                store.checkGroups[CheckType.TrainType].checkAll = false;
                store.checkGroups[CheckType.TrainType].checkedList = [];
                
                store.onCheckAllBoxChange(CheckType.TrainType);
                
                expect(store.checkGroups[CheckType.TrainType].checkAll).toBe(true);
                expect(store.checkGroups[CheckType.TrainType].checkedList).toEqual(
                    store.checkGroups[CheckType.TrainType].options
                );
                expect(store.checkGroups[CheckType.TrainType].indeterminate).toBe(false);
            });

            it("deselects all options when unchecking all", () => {
                store.checkGroups[CheckType.TrainType].checkAll = true;
                store.checkGroups[CheckType.TrainType].checkedList = store.checkGroups[CheckType.TrainType].options;
                
                store.onCheckAllBoxChange(CheckType.TrainType);
                
                expect(store.checkGroups[CheckType.TrainType].checkAll).toBe(false);
                expect(store.checkGroups[CheckType.TrainType].checkedList).toEqual([]);
                expect(store.checkGroups[CheckType.TrainType].indeterminate).toBe(false);
            });

            it("handles invalid check type gracefully", () => {
                expect(() => store.onCheckAllBoxChange(999)).not.toThrow();
            });
        });

        describe("time conversion methods", () => {
            it("converts minutes to dayjs range correctly", () => {
                const result = store.minutesToDayjsRange([60, 120]);
                expect(result[0].format("HH:mm")).toBe("01:00");
                expect(result[1].format("HH:mm")).toBe("02:00");
            });

            it("converts dayjs range to minutes correctly", () => {
                const start: Dayjs = dayjs("2025-09-01 08:30");
                const end: Dayjs = dayjs("2025-09-01 18:45");
                expect(start.isValid()).toBe(true);
                expect(end.isValid()).toBe(true);
                const result = store.dayjsRangeToMinutes([start, end]);
                expect(result).toEqual([510, 1125]);
            });
        });

        describe("onSliderChange", () => {
            it("updates start time range", () => {
                store.onSliderChange("start", [60, 120]);
                expect(store.startTimeRangeNumber).toEqual([60, 120]);
                expect(store.startTimeRange[0].format("HH:mm")).toBe("01:00");
                expect(store.startTimeRange[1].format("HH:mm")).toBe("02:00");
            });

            it("updates end time range", () => {
                store.onSliderChange("end", [480, 600]);
                expect(store.endTimeRangeNumber).toEqual([480, 600]);
                expect(store.endTimeRange[0].format("HH:mm")).toBe("08:00");
                expect(store.endTimeRange[1].format("HH:mm")).toBe("10:00");
            });
        });

        describe("onTimePickerChange", () => {
            it("updates start time range", () => {
                const start = dayjs("2025-09-01 09:00");
                const end = dayjs("2025-09-01 17:00");
                store.onTimePickerChange("start", [start, end]);
                
                expect(store.startTimeRange).toEqual([start, end]);
                expect(store.startTimeRangeNumber).toEqual([540, 1020]);
            });

            it("updates end time range", () => {
                const start = dayjs("2025-09-01 10:00");
                const end = dayjs("2025-09-01 20:00");
                store.onTimePickerChange("end", [start, end]);
                
                expect(store.endTimeRange).toEqual([start, end]);
                expect(store.endTimeRangeNumber).toEqual([600, 1200]);
            });
        });

        describe("resetTimeRange", () => {
            it("resets start time range to default", () => {
                store.startTimeRange = [dayjs("2025-09-01 08:00"), dayjs("2025-09-01 18:00")];
                store.startTimeRangeNumber = [480, 1080];
                
                store.resetTimeRange("start");
                
                expect(store.startTimeRange[0].format("HH:mm")).toBe("00:00");
                expect(store.startTimeRange[1].format("HH:mm")).toBe("23:59");
                expect(store.startTimeRangeNumber).toEqual([0, 1439]);
            });

            it("resets end time range to default", () => {
                store.endTimeRange = [dayjs("2025-09-01 08:00"), dayjs("2025-09-01 18:00")];
                store.endTimeRangeNumber = [480, 1080];
                
                store.resetTimeRange("end");
                
                expect(store.endTimeRange[0].format("HH:mm")).toBe("00:00");
                expect(store.endTimeRange[1].format("HH:mm")).toBe("23:59");
                expect(store.endTimeRangeNumber).toEqual([0, 1439]);
            });
        });

        describe("toggleSortType", () => {
            it("toggles sort order when same type is selected", () => {
                store.sortType = SortType.DepartureTime;
                store.sortOrderAsc = true;
                
                store.toggleSortType(SortType.DepartureTime);
                
                expect(store.sortType).toBe(SortType.DepartureTime);
                expect(store.sortOrderAsc).toBe(false);
            });

            it("changes sort type and resets to ascending", () => {
                store.sortType = SortType.DepartureTime;
                store.sortOrderAsc = false;
                
                store.toggleSortType(SortType.TravelTime);
                
                expect(store.sortType).toBe(SortType.TravelTime);
                expect(store.sortOrderAsc).toBe(true);
            });
        });

        describe("resetSpecificState", () => {
            it("resets all filter states to default", () => {
                // 修改一些状态
                store.onlyShowAvailable = true;
                store.checkGroups[CheckType.SeatType].options = ["二等座", "一等座"];
                
                store.resetSpecificState();
                
                expect(store.onlyShowAvailable).toBe(false);
                expect(store.checkGroups[CheckType.SeatType].options).toEqual(['加载中...']);
                expect(store.checkGroups[CheckType.SeatType].checkedList).toEqual(['加载中...']);
                expect(store.checkGroups[CheckType.SeatType].checkAll).toBe(true);
            });
        });

        describe("querySchedule validation", () => {
            it("shows error when query date is empty", async () => {
                const antdModule = await import("ant-design-vue");
                store.queryDate = "";
                await store.querySchedule();
                expect(antdModule.message.error).toHaveBeenCalledWith("请填写查询日期");
            });

            it("shows error when departure text is empty", async () => {
                const antdModule = await import("ant-design-vue");
                store.queryDate = "2024-06-01";
                store.queryDepartureText = "";
                store.queryArrivalText = "上海";
                await store.querySchedule();
                expect(antdModule.message.error).toHaveBeenCalledWith("请填写出发地点和到达地点");
            });

            it("shows error when arrival text is empty", async () => {
                const antdModule = await import("ant-design-vue");
                store.queryDate = "2024-06-01";
                store.queryDepartureText = "北京";
                store.queryArrivalText = "";
                await store.querySchedule();
                expect(antdModule.message.error).toHaveBeenCalledWith("请填写出发地点和到达地点");
            });

            it("shows error for invalid departure text format", async () => {
                const antdModule = await import("ant-design-vue");
                const generalModule = await import("@/stores/general");
                const generalStore = generalModule.useGeneralStore();
                generalStore.checkInputString.mockReturnValueOnce(undefined);
                
                store.queryDate = "2024-06-01";
                store.queryDepartureText = "invalid";
                store.queryArrivalText = "上海";
                
                await store.querySchedule();
                expect(antdModule.message.error).toHaveBeenCalledWith("出发地点或到达地点格式不正确，请检查输入");
            });
        });

    });

    describe("edge cases", () => {
        it("handles empty query result gracefully", () => {
            store.queryResult = [];
            expect(store.displaySchedules).toEqual([]);
        });

        it("handles missing seat types in schedule", () => {
            const scheduleWithoutSeatTypes = [{
                trainNumber: "G1",
                departureStation: "北京南站",
                arrivalStation: "上海虹桥站"
            }];
            store.queryResult = scheduleWithoutSeatTypes;
            expect(() => store.displaySchedules).not.toThrow();
        });

        it("handles time range edge values", () => {
            store.onSliderChange("start", [0, 1439]);
            expect(store.startTimeRange[0].format("HH:mm")).toBe("00:00");
            expect(store.startTimeRange[1].format("HH:mm")).toBe("23:59");
        });

        it("maintains state consistency after multiple operations", () => {
            store.toggleSortType(SortType.Price);
            store.toggleSortType(SortType.Price);
            store.resetTimeRange("start");
            store.onCheckAllBoxChange(CheckType.TrainType);
            
            expect(store.sortType).toBe(SortType.Price);
            expect(store.sortOrderAsc).toBe(false);
            expect(store.startTimeRangeNumber).toEqual([0, 1439]);
        });
    });

    describe("integration scenarios", () => {

        beforeEach(() => {
            store.queryDate = "2025-09-01";
        })

        it("complete filter and sort workflow", () => {
            // Set up data
            store.queryResult = mockDirectScheduleData.data.solutions;
            store.queryMode = "direct";
            
            // Apply filters
            store.onlyShowAvailable = true;
            store.checkGroups[CheckType.TrainType].checkedList = ["G/C"];
            store.toggleSortType(SortType.DepartureTime);
            
            const result = store.displaySchedules;
            expect(result).toHaveLength(1);
            expect((result[0] as any).trainNumber).toBe("G1");
        });

        it("time range filtering workflow", () => {
            store.queryResult = mockDirectScheduleData.data.solutions;
            store.queryMode = "direct";
            
            // Set time range to exclude some trains
            store.onSliderChange("start", [480, 1439]); // From 08:00
            
            const result = store.displaySchedules;
            expect(result.every((schedule: any) => schedule.departureTime >= "08:00")).toBe(true);
        });
    });
});