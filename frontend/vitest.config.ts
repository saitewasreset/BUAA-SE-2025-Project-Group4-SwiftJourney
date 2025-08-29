import { fileURLToPath } from 'node:url'
import { mergeConfig, defineConfig, configDefaults } from 'vitest/config'
import viteConfig from './vite.config'

export default mergeConfig(
  viteConfig,
  defineConfig({
    test: {
      environment: 'jsdom',
      exclude: [...configDefaults.exclude, 'e2e/**'],
      root: fileURLToPath(new URL('./', import.meta.url)),
      setupFiles: "test/setup.ts",
      coverage: {
        provider: 'istanbul', // or 'istanbul'
        enabled: true,
        include: ['src/**/*.{ts,tsx}']
      },
    },
  }),
)
