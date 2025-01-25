import { defineConfig } from '@farmfe/core';

export default defineConfig({
  plugins: ['@farmfe/plugin-react'],
  compilation: {
    input: {
      index: "./page/index.html"
    }
  },
  server: {
    proxy: {
      '/api': {
        target: process.env.API_ORIGIN || "http://localhost:8080",
        changeOrigin: true,
      }
    }
  }
});
