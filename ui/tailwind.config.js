/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'bg-deep': '#050508',
        'bg-surface': '#0d0d14',
        'bg-elevated': '#151520',
        'bg-glass': 'rgba(13, 13, 20, 0.72)',
        'tactical-cyan': '#00f2ff',
        'hazard-orange': '#ff8800',
        'stealth-gray': '#94a3b8',
        'terminal-green': '#00ff95',
        'terminal-emerald': '#10b981',
        'text-dim': '#808095',
      },
      fontFamily: {
        'ui': ['Inter', 'system-ui', 'sans-serif'],
        'data': ['JetBrains Mono', 'monospace'],
      },
      spacing: {
        'unit': '4px',
        '18': '4.5rem',
      },
      borderRadius: {
        'xs': '2px',
        'sm': '6px',
        'md': '10px',
        'lg': '14px',
        'xl': '18px',
        '2xl': '24px',
      },
      animation: {
        'shimmer': 'shimmer 4s infinite',
      },
      keyframes: {
        shimmer: {
          '0%': { 'mask-position': '200%' },
          '100%': { 'mask-position': '-200%' },
        }
      }
    },
  },
  plugins: [],
}

