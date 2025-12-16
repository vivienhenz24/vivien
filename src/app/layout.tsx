
import type { Metadata } from "next";
import { Canary } from '@fuzzycanary/core/react'
import "./globals.css";
import AnalyticsWrapper from './analytics'

export const metadata: Metadata = {
  title: "I hope you enjoy these",
  description: "just a diary",
  icons: {
    icon: '/globe.svg',
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className="antialiased text-gray-900">
        <Canary />
        {children}
        <AnalyticsWrapper />
      </body>
    </html>
  );
}
