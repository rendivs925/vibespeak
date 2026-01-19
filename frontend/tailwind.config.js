/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.rs",
    "./index.html",
    "./src/**/*.html",
  ],
  theme: {
    extend: {
      colors: {
        // Indigo (Brand Primary) - Use sparingly for active states, focus rings, and primary CTAs
        indigo: {
          50: '#eef2ff',  // Soft backgrounds, hover states
          100: '#e0e7ff', // Subtle accents
          500: '#6366f1', // Primary brand
          600: '#4f46e5', // Primary buttons, active states
          700: '#4338ca', // Button hover, dark accents
          900: '#312e81', // Text on active elements
        },
        // Slate (Neutral Primary) - The foundation of the UI - borders, text, backgrounds
        slate: {
          50: '#f8fafc',   // Page backgrounds, subtle tints
          100: '#f1f5f9',  // Card backgrounds, hover states
          200: '#e2e8f0',  // Borders, dividers
          300: '#cbd5e1',  // Hover borders
          400: '#94a3b8',  // Placeholder text, icons
          500: '#64748b',  // Secondary text
          600: '#475569',  // Body text secondary
          700: '#334155',  // Body text
          900: '#0f172a',  // Headings, emphasis
        },
        // Gray (Alternative Neutral) - Interchangeable with Slate for variety
        gray: {
          400: '#9ca3af', // Icons, placeholders
          500: '#6b7280', // Secondary text
          600: '#4b5563', // Body text
          700: '#374151', // Emphasis text
          900: '#111827', // Headings
        },
        // Semantic Colors - Success (Emerald/Green)
        emerald: {
          50: '#ecfdf5',  // Success backgrounds
          200: '#a7f3d0', // Success borders
          500: '#10b981', // Success indicators
          700: '#047857', // Success text
        },
        // Warning (Amber/Yellow)
        amber: {
          50: '#fffbeb',  // Warning backgrounds
          200: '#fde68a', // Warning borders
          500: '#f59e0b', // Warning indicators
          600: '#d97706', // Warning text
        },
        // Error (Red)
        red: {
          50: '#fef2f2',  // Error backgrounds
          100: '#fee2e2', // Error border light
          200: '#fecaca', // Error borders
          600: '#dc2626', // Error icons
          700: '#b91c1c', // Error text
        },
        // Info (Blue)
        blue: {
          50: '#eff6ff',  // Info backgrounds
          600: '#2563eb', // Info indicators
        },
      },
      boxShadow: {
        'sm': '0 1px 2px 0 rgb(0 0 0 / 0.05)',
        'md': '0 4px 6px -1px rgb(0 0 0 / 0.1)',
        'lg': '0 10px 15px -3px rgb(0 0 0 / 0.1)',
        'xl': '0 20px 25px -5px rgb(0 0 0 / 0.1)',
        '2xl': '0 25px 50px -12px rgb(0 0 0 / 0.25)',
        // Design system shadow scale
        'ds-sm': '0 1px 2px 0 rgb(241 245 249 / 0.3)',     // Extra soft
        'ds-default': '0 1px 2px 0 rgb(241 245 249 / 0.5)',  // Soft
        'ds-md': '0 4px 6px -1px rgb(241 245 249 / 0.6)',    // Medium
        'ds-lg': '0 10px 15px -3px rgb(165 180 252 / 0.5)',  // Large (branded)
        'ds-xl': '0 20px 25px -5px rgb(241 245 249 / 0.5)',  // Extra large
        'ds-2xl': '0 25px 50px -12px rgb(241 245 249 / 0.5)', // 2X Large
      },
      fontFamily: {
        'sans': ['Inter', 'system-ui', '-apple-system', 'BlinkMacSystemFont', '"Segoe UI"', 'Roboto', 'sans-serif'],
        'mono': ['ui-monospace', '"SF Mono"', 'Menlo', 'Monaco', '"Courier New"', 'monospace'],
      },
      fontSize: {
        'xs': ['0.75rem', { lineHeight: '1rem' }],      // 12px - Captions, metadata
        'sm': ['0.875rem', { lineHeight: '1.25rem' }],  // 14px - Body text, inputs
        'base': ['1rem', { lineHeight: '1.5rem' }],     // 16px - Default body
        'lg': ['1.125rem', { lineHeight: '1.75rem' }],  // 18px - Section headers
        'xl': ['1.25rem', { lineHeight: '1.75rem' }],   // 20px - Card titles
        '2xl': ['1.5rem', { lineHeight: '2rem' }],      // 24px - Page headers
        '3xl': ['1.875rem', { lineHeight: '2.25rem' }], // 30px - Main page titles
      },
      fontWeight: {
        'normal': '400',   // Body text
        'medium': '500',   // Labels, emphasis
        'semibold': '600', // Headings, buttons
        'bold': '700',     // Strong emphasis (use sparingly)
      },
      borderRadius: {
        'none': '0',
        'sm': '2px',
        'DEFAULT': '4px',
        'md': '6px',
        'lg': '8px',       // Small elements
        'xl': '12px',      // Inputs, buttons
        '2xl': '16px',     // Cards, sections
        'full': '9999px',  // Circles, pills
      },
      spacing: {
        // Design system spacing scale (4px increments)
        '0': '0px',
        '0.5': '2px',   // 0.125rem
        '1': '4px',     // 0.25rem
        '1.5': '6px',   // 0.375rem
        '2': '8px',     // 0.5rem
        '2.5': '10px',  // 0.625rem
        '3': '12px',    // 0.75rem
        '4': '16px',    // 1rem
        '5': '20px',    // 1.25rem
        '6': '24px',    // 1.5rem
        '7': '28px',    // 1.75rem
        '8': '32px',    // 2rem
        '10': '40px',   // 2.5rem
        '12': '48px',   // 3rem
        '16': '64px',   // 4rem
        '20': '80px',   // 5rem
      },
      transitionDuration: {
        '100': '100ms',  // Instant feedback (rare)
        '200': '200ms',  // Standard interactions
        '300': '300ms',  // Smooth expansions
        '500': '500ms',  // Slow, emphasize
        '700': '700ms',  // Progress indicators
      },
      animation: {
        'spin-slow': 'spin 3s linear infinite',
      },
    },
  },
  plugins: [
    require('@tailwindcss/forms')({
      strategy: 'class',
    }),
  ],
}