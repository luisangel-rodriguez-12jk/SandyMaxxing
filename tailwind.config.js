/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        mint: {
          50: "#f1f7f4",
          100: "#dcece4",
          200: "#b9d9c9",
          300: "#8ec0a7",
          400: "#63a485",
          500: "#428a6a",
          600: "#306d54",
          700: "#265744",
          800: "#1f4636",
          900: "#163126",
        },
      },
      fontFamily: {
        sans: ["Inter", "ui-sans-serif", "system-ui", "sans-serif"],
      },
      boxShadow: {
        card: "0 1px 2px rgba(0,0,0,0.04), 0 4px 16px rgba(16,24,40,0.06)",
      },
    },
  },
  plugins: [],
};
