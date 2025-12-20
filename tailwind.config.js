/** @type {import('tailwindcss').Config} */
export default {
    content: [
        "./index.html",
        "./src/**/*.{vue,js,ts,jsx,tsx}",
    ],
    theme: {
        extend: {
            colors: {
                'brut-black': '#000000',
                'brut-white': '#FFFFFF',
                'brut-gray': '#CCCCCC',
                ink: '#1a1a1a',      // Soft Black
                paper: '#f4f4f0',    // Warm Off-White
                stone: '#e5e5e0',    // Separation Grey
                accent: '#ff5f1f',   // Vibrant Orange
            },
            fontFamily: {
                mono: ['"Courier New"', 'monospace'],
                sans: ['Inter', 'sans-serif'],
                serif: ['"Playfair Display"', 'Georgia', 'serif'], // Added for editorial touches
            },
            boxShadow: {
                'brut': '4px 4px 0px 0px rgba(0,0,0,1)',
            }
        },
    },
    plugins: [],
}
