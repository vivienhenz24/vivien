'use client'
import type { Metadata } from "next";
import { Analytics } from '@vercel/analytics/react';
import '@fuzzycanary/core/auto'
import "./globals.css";


export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className="antialiased text-gray-900">
        {children}
        <Analytics />
      </body>
    </html>
  );
}
