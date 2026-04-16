import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: { "@": path.resolve(__dirname, "./src") },
  },
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  // CVE-2023-46115 : ne PAS exposer les variables d'env TAURI_ au frontend
  envPrefix: ["VITE_"],
});
