/** @type {import('tailwindcss').Config} */
export default {
    content: ['./src/**/*.{astro,html,js,jsx,md,mdx,svelte,ts,tsx,vue}'],
    theme: {
        extend: {
            colors: {
                primary: {
                    DEFAULT: '#667eea',
                    dark: '#5568d3',
                },
                secondary: '#764ba2',
            },
        },
    },
    plugins: [],
}
