import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import router, { globalBeforeEachGuard } from '../../src/router/index'
import { RouteLocationNormalized } from 'vue-router'

// Mock dependencies
vi.mock('@/stores/user', () => ({
  useUserStore: vi.fn(() => ({
    restoreUserFromCookie: vi.fn()
  }))
}))

vi.mock('ant-design-vue', () => ({
  message: {
    warning: vi.fn(),
    info: vi.fn()
  }
}))

vi.mock('vue', async () => {
  const actual = await vi.importActual('vue')
  return {
    ...actual,
    nextTick: vi.fn()
  }
})

// Mock localStorage
const mockLocalStorage = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn()
}

Object.defineProperty(window, 'localStorage', {
  value: mockLocalStorage
})

// Helper function to create mock route
function createMockRoute(path: string, fullPath?: string): RouteLocationNormalized {
  return {
    path,
    fullPath: fullPath || path,
    name: undefined,
    params: {},
    query: {},
    hash: '',
    matched: [],
    meta: {},
    redirectedFrom: undefined
  } as RouteLocationNormalized
}

describe('Router', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.resetAllMocks()
  })

  describe('Router Configuration', () => {
    it('should have correct base configuration', () => {
      expect(router.options.history).toBeDefined()
      expect(router.options.routes).toHaveLength(11)
    })

    it('should have correct route paths', () => {
      const routes = router.options.routes
      const paths = routes.map(route => route.path)
      
      expect(paths).toContain('/')
      expect(paths).toContain('/homepage')
      expect(paths).toContain('/trainTicket')
      expect(paths).toContain('/hotel')
      expect(paths).toContain('/meal')
      expect(paths).toContain('/hotel/:id')
      expect(paths).toContain('/login')
      expect(paths).toContain('/register')
      expect(paths).toContain('/personalhomepage/:activeIndex')
      expect(paths).toContain('/trainTransaction')
      expect(paths).toContain('/paytransaction/:transactionId')
    })

    it('should have root path redirect to homepage', () => {
      const rootRoute = router.options.routes.find(route => route.path === '/')
      expect(rootRoute?.redirect).toBe('/homepage')
    })

    it('should have hotel detail route with props enabled', () => {
      const hotelDetailRoute = router.options.routes.find(route => route.path === '/hotel/:id')
      expect(hotelDetailRoute?.props).toBe(true)
      expect(hotelDetailRoute?.name).toBe('hotelDetail')
    })

    it('should have personal homepage route with correct meta', () => {
      const personalRoute = router.options.routes.find(route => route.path === '/personalhomepage/:activeIndex')
      expect(personalRoute?.meta).toEqual({
        requiresAuth: true,
        forceRefresh: true
      })
    })

    it('should have pay page route with props function', () => {
      const payRoute = router.options.routes.find(route => route.path === '/paytransaction/:transactionId')
      expect(typeof payRoute?.props).toBe('function')
    })
  })

  describe('Navigation Guard - Unauthenticated Users', () => {
    beforeEach(() => {
      mockLocalStorage.getItem.mockReturnValue('false')
    })

    it('should redirect to login when accessing protected route', async () => {
      const { message } = await import('ant-design-vue')
      const to = createMockRoute('/homepage')
      const from = createMockRoute('/login')
      const next = vi.fn()

  await globalBeforeEachGuard(to, from, next)

      expect(message.warning).toHaveBeenCalledWith('请先登录')
      expect(next).toHaveBeenCalledWith({ 
        path: '/login', 
        query: { redirect: '/homepage' } 
      })
    })

    it('should allow access to login page', async () => {
      const to = createMockRoute('/login')
      const from = createMockRoute('/homepage')
      const next = vi.fn()

  await globalBeforeEachGuard(to, from, next)

      expect(next).toHaveBeenCalledWith()
    })

    it('should allow access to register page', async () => {
      const to = createMockRoute('/register')
      const from = createMockRoute('/homepage')
      const next = vi.fn()

  await globalBeforeEachGuard(to, from, next)

      expect(next).toHaveBeenCalledWith()
    })
  })

  describe('Navigation Guard - Authenticated Users', () => {
    beforeEach(() => {
      mockLocalStorage.getItem.mockReturnValue('true')
    })

    it('should redirect to homepage when accessing login page', async () => {
      const { message } = await import('ant-design-vue')
      const to = createMockRoute('/login')
      const from = createMockRoute('/homepage')
      const next = vi.fn()

  await globalBeforeEachGuard(to, from, next)

      expect(message.info).toHaveBeenCalledWith('您已登录')
      expect(next).toHaveBeenCalledWith({ path: '/homepage' })
    })

    it('should redirect to homepage when accessing register page', async () => {
      const { message } = await import('ant-design-vue')
      const to = createMockRoute('/register')
      const from = createMockRoute('/homepage')
      const next = vi.fn()

  await globalBeforeEachGuard(to, from, next)

      expect(message.info).toHaveBeenCalledWith('您已登录')
      expect(next).toHaveBeenCalledWith({ path: '/homepage' })
    })

    it('should restore user data when accessing personal data page', async () => {
      const { useUserStore } = await import('../../src/stores/user')
      const mockRestoreUser = vi.fn();
      (useUserStore as any).mockReturnValue({
        restoreUserFromCookie: mockRestoreUser
      })

      const to = createMockRoute('/personalhomepage/personaldata')
      const from = createMockRoute('/homepage')
      const next = vi.fn()

  await globalBeforeEachGuard(to, from, next)

      expect(mockRestoreUser).toHaveBeenCalledWith(router)
      expect(next).toHaveBeenCalledWith()
    })

    it('should call nextTick for other routes', async () => {
      const { nextTick } = await import('vue')
      const to = createMockRoute('/homepage')
      const from = createMockRoute('/login')
      const next = vi.fn()

  await globalBeforeEachGuard(to, from, next)

      expect(nextTick).toHaveBeenCalled()
      expect(next).toHaveBeenCalledWith()
    })
  })

  describe('Pay Page Route Props', () => {
    it('should correctly transform route params and query to props', () => {
      const payRoute = router.options.routes.find(route => route.path === '/paytransaction/:transactionId')
      const propsFunction = payRoute?.props as Function
      
      const mockRoute = {
        params: { transactionId: '123' },
        query: { money: '100' }
      }

      const props = propsFunction(mockRoute)
      expect(props).toEqual({
        transactionId: '123',
        money: '100'
      })
    })
  })
})