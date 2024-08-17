import { JetBrains_Mono } from "next/font/google";
import localFont from "next/font/local";

export const jetbrains = JetBrains_Mono({
  subsets: ["latin"],
  display: "swap",
});
export const f1 = localFont({
  src: [
    {
      path: "./fonts/Formula1-Display-Bold.woff2",
      weight: "400",
      style: "normal",
    },
  ],
  display: "swap",
});
