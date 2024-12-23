import { defineConfig } from 'vite';

export default defineConfig(({ mode }) => {
  const isProduction = mode === 'production';

  return {
    root: './internal/ts-client',
    build: {
      lib: {
        entry: 'client.ts',
        name: 'client',
        fileName: 'client'
      },
      outDir: '../static/build',     // Output directory relative to the root
      emptyOutDir: true,          // Clear the output directory before building
      minify: isProduction,       // We want to keep client-side code human readable, 
      sourcemap: isProduction,    // but still want to minify for performance. 
    },                            // Source maps solve this issue
    resolve: {
      alias: {
          // If you're using npm-installed htmx or other libraries
          htmx: 'htmx.org', // Example alias (customize as needed)
      },
    },
    server: {
      port: 8081,                 // Development server port
      strictPort: true,           // Fail if the port is unavailable
      open: false,
      proxy: {
        // Proxy API requests to your Go server
        '/api': 'http://localhost:8080',
      },
    },
  }
});
