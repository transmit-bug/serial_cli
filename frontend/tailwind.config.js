/** @type {import('tailwindcss').Config} */
export default {
  darkMode: ['class'],
  content: [
    './index.html',
    './src/**/*.{js,ts,jsx,tsx}',
  ],
  theme: {
    extend: {
      colors: {
        // shadcn/ui semantic colors (mapped to Cyber-Industrial theme)
        primary: {
          DEFAULT: '#00ff41',
          foreground: '#000000',
        },
        secondary: {
          DEFAULT: '#18181b',
          foreground: '#fafafa',
        },
        destructive: {
          DEFAULT: '#ff4757',
          foreground: '#fafafa',
        },
        muted: {
          DEFAULT: '#18181b',
          foreground: '#a0a0a8',
        },
        accent: {
          DEFAULT: '#00ff41',
          foreground: '#000000',
        },
        popover: {
          DEFAULT: '#0d0d0f',
          foreground: '#fafafa',
        },
        card: {
          DEFAULT: '#0d0d0f',
          foreground: '#fafafa',
        },
        border: 'rgba(255, 255, 255, 0.1)',
        input: 'rgba(255, 255, 255, 0.1)',
        ring: '#00ff41',

        // Original Cyber-Industrial colors
        signal: {
          DEFAULT: '#00ff41',
          dim: 'rgba(0, 255, 65, 0.08)',
          glow: 'rgba(0, 255, 65, 0.15)',
        },
        amber: {
          DEFAULT: '#ffb142',
          dim: 'rgba(255, 177, 66, 0.08)',
        },
        alert: {
          DEFAULT: '#ff4757',
          dim: 'rgba(255, 71, 87, 0.08)',
        },
        info: {
          DEFAULT: '#53a0fd',
          dim: 'rgba(83, 160, 253, 0.08)',
        },
        bg: {
          deepest: '#0d0d0f',
          deep: '#121214',
          base: '#18181b',
          elevated: '#1c1c1f',
          floating: '#222226',
        },
        borderCustom: {
          DEFAULT: 'rgba(255, 255, 255, 0.1)',
          subtle: 'rgba(255, 255, 255, 0.06)',
          strong: 'rgba(255, 255, 255, 0.15)',
        },
        text: {
          primary: '#fafafa',
          secondary: '#a0a0a8',
          tertiary: '#5a5a60',
        },
      },
      fontFamily: {
        sans: ['Instrument Sans', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'SF Mono', 'monospace'],
        display: ['Instrument Serif', 'Georgia', 'serif'],
      },
      fontSize: {
        'xs': ['0.75rem', { lineHeight: '1rem' }],
        'sm': ['0.875rem', { lineHeight: '1.25rem' }],
        'base': ['1rem', { lineHeight: '1.5rem' }],
        'lg': ['1.125rem', { lineHeight: '1.75rem' }],
        'xl': ['1.25rem', { lineHeight: '1.75rem' }],
        '2xl': ['1.5rem', { lineHeight: '2rem' }],
      },
      spacing: {
        '1': '4px',
        '2': '8px',
        '3': '12px',
        '4': '16px',
        '5': '20px',
        '6': '24px',
        '8': '32px',
        '10': '40px',
        '12': '48px',
        '16': '64px',
      },
      borderRadius: {
        'sm': '2px',
        'md': '4px',
        'lg': '6px',
        'xl': '12px',
        'full': '9999px',
        // shadcn/ui radius variable
        'radiust': '0.25rem',
      },
      boxShadow: {
        'sm': '0 1px 2px rgba(0, 0, 0, 0.4)',
        'md': '0 4px 8px rgba(0, 0, 0, 0.4)',
        'lg': '0 8px 24px rgba(0, 0, 0, 0.5)',
        'xl': '0 16px 48px rgba(0, 0, 0, 0.6)',
      },
      animation: {
        'fade-in': 'fadeIn 200ms cubic-bezier(0.215, 0.61, 0.355, 1)',
        'slide-up': 'slideUp 320ms cubic-bezier(0.215, 0.61, 0.355, 1)',
        'shimmer': 'shimmer 2s infinite',
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
      },
      keyframes: {
        fadeIn: {
          from: { opacity: '0' },
          to: { opacity: '1' },
        },
        slideUp: {
          from: { opacity: '0', transform: 'translateY(8px)' },
          to: { opacity: '1', transform: 'translateY(0)' },
        },
        shimmer: {
          '0%': { backgroundPosition: '-200% 0' },
          '100%': { backgroundPosition: '200% 0' },
        },
      },
    },
  },
  plugins: [require("tailwindcss-animate")],
}
