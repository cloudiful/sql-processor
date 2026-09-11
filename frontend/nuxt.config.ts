export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  ssr: false,
  srcDir: 'src/',
  devtools: { enabled: false },
  modules: [
    ['@nuxt/ui', { fonts: false, colorMode: false }],
    '@nuxtjs/i18n',
  ],
  css: ['~/style.css'],
  typescript: {
    strict: true,
    typeCheck: true,
  },
  i18n: {
    defaultLocale: 'zh-CN',
    locales: ['zh-CN', 'en'],
    strategy: 'no_prefix',
  },
  nitro: {
    preset: 'static',
    devProxy: {
      '/api': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
    },
  },
  devServer: {
    port: 5173,
  },
  routeRules: {
    '/**': { prerender: true },
  },
})
