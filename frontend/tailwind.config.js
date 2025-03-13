/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: ["class"],
  content: [
    "./pages/**/*.{ts,tsx}",
    "./components/**/*.{ts,tsx}",
    "./app/**/*.{ts,tsx}",
    "./src/**/*.{ts,tsx}",
  ],
  safelist: [
    // General UI colors
    // red
    "text-red-400",
    "text-red-500",
    "text-red-600",
    "text-red-700",
    "bg-red-400",
    "bg-red-500",
    "bg-red-600",
    "bg-red-700",
    "fill-red-400",
    "fill-red-500",
    "fill-red-600",
    "fill-red-700",
    "stroke-red-400",
    "stroke-red-500",
    "stroke-red-600",
    "stroke-red-700",
    // green
    "text-green-400",
    "text-green-500",
    "text-green-600",
    "text-green-700",
    "bg-green-400",
    "bg-green-500",
    "bg-green-600",
    "bg-green-700",
    "fill-green-400",
    "fill-green-500",
    "fill-green-600",
    "fill-green-700",
    "stroke-green-400",
    "stroke-green-500",
    "stroke-green-600",
    "stroke-green-700",
    // blue
    "text-blue-400",
    "text-blue-500",
    "text-blue-600",
    "text-blue-700",
    "bg-blue-400",
    "bg-blue-500",
    "bg-blue-600",
    "bg-blue-700",
    "fill-blue-400",
    "fill-blue-500",
    "fill-blue-600",
    "fill-blue-700",
    "stroke-blue-400",
    "stroke-blue-500",
    "stroke-blue-600",
    "stroke-blue-700",
    // orange
    "text-orange-400",
    "text-orange-500",
    "text-orange-600",
    "text-orange-700",
    "bg-orange-400",
    "bg-orange-500",
    "bg-orange-600",
    "bg-orange-700",
    "fill-orange-400",
    "fill-orange-500",
    "fill-orange-600",
    "fill-orange-700",
    "stroke-orange-400",
    "stroke-orange-500",
    "stroke-orange-600",
    "stroke-orange-700",
    // purple
    "text-purple-400",
    "text-purple-500",
    "text-purple-600",
    "text-purple-700",
    "bg-purple-400",
    "bg-purple-500",
    "bg-purple-600",
    "bg-purple-700",
    "fill-purple-400",
    "fill-purple-500",
    "fill-purple-600",
    "fill-purple-700",
    "stroke-purple-400",
    "stroke-purple-500",
    "stroke-purple-600",
    "stroke-purple-700",

    // Tag colors
    "bg-slate-400",
    "bg-slate-600",
    "bg-slate-900",
    //
    "bg-gray-400",
    "bg-gray-600",
    "bg-gray-900",
    //
    "bg-zinc-400",
    "bg-zinc-600",
    "bg-zinc-900",
    //
    "bg-neutral-400",
    "bg-neutral-600",
    "bg-neutral-900",
    //
    "bg-stone-400",
    "bg-stone-600",
    "bg-stone-900",
    //
    "bg-red-400",
    "bg-red-600",
    "bg-red-900",
    //
    "bg-orange-400",
    "bg-orange-600",
    "bg-orange-900",
    //
    "bg-amber-400",
    "bg-amber-600",
    "bg-amber-900",
    //
    "bg-yellow-400",
    "bg-yellow-600",
    "bg-yellow-900",
    //
    "bg-lime-400",
    "bg-lime-600",
    "bg-lime-900",
    //
    "bg-green-400",
    "bg-green-600",
    "bg-green-900",
    //
    "bg-emerald-400",
    "bg-emerald-600",
    "bg-emerald-900",
    //
    "bg-teal-400",
    "bg-teal-600",
    "bg-teal-900",
    //
    "bg-cyan-400",
    "bg-cyan-600",
    "bg-cyan-900",
    //
    "bg-sky-400",
    "bg-sky-600",
    "bg-sky-900",
    //
    "bg-blue-400",
    "bg-blue-600",
    "bg-blue-900",
    //
    "bg-indigo-400",
    "bg-indigo-600",
    "bg-indigo-900",
    //
    "bg-violet-400",
    "bg-violet-600",
    "bg-violet-900",
    //
    "bg-purple-400",
    "bg-purple-600",
    "bg-purple-900",
    //
    "bg-fuchsia-400",
    "bg-fuchsia-600",
    "bg-fuchsia-900",
    //
    "bg-pink-400",
    "bg-pink-600",
    "bg-pink-900",
    //
    "bg-rose-400",
    "bg-rose-600",
    "bg-rose-900",
  ],
  prefix: "",
  theme: {
    container: {
      center: true,
      padding: "2rem",
      screens: {
        "2xl": "1680px",
      },
    },
    extend: {
      colors: {
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive))",
          foreground: "hsl(var(--destructive-foreground))",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        popover: {
          DEFAULT: "hsl(var(--popover))",
          foreground: "hsl(var(--popover-foreground))",
        },
        card: {
          DEFAULT: "hsl(var(--card))",
          foreground: "hsl(var(--card-foreground))",
        },
      },
      borderRadius: {
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
      },
      keyframes: {
        "accordion-down": {
          from: { height: "0" },
          to: { height: "var(--radix-accordion-content-height)" },
        },
        "accordion-up": {
          from: { height: "var(--radix-accordion-content-height)" },
          to: { height: "0" },
        },
      },
      animation: {
        "accordion-down": "accordion-down 0.2s ease-out",
        "accordion-up": "accordion-up 0.2s ease-out",
      },
    },
  },
  plugins: [require("tailwindcss-animate")],
};
