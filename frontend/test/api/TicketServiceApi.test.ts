import { TicketServiceApi } from '../../src/api/TicketServiceApi/TicketServiceApi';
import { describe, it, expect } from 'vitest';

import type {
//   scheduleRequest,
//   trainTransactionRequest,
//   TrainInfoQuery,
} from '../../src/interface/ticketServiceInterface'

describe('TicketServiceApi', () => {
    it('queryDirectSchedule should return a promise', () => {
        const result = TicketServiceApi.queryDirectSchedule({});
        expect(result).toBeInstanceOf(Promise);
    });

    it('queryIndirectSchedule should return a promise', () => {
        const result = TicketServiceApi.queryIndirectSchedule({});
        expect(result).toBeInstanceOf(Promise);
    });

    it('trainSchedule should return a promise', () => {
        const result = TicketServiceApi.trainSchedule({});
        expect(result).toBeInstanceOf(Promise);
    });

    it('submitOrder should return a promise', () => {
        const result = TicketServiceApi.submitOrder({});
        expect(result).toBeInstanceOf(Promise);
    });
});
