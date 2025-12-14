
import type { Metadata } from "next";
import { renderCanaryComment, getCanaryPayload } from '@fuzzycanary/core'
import '@fuzzycanary/core/auto'
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
  const payload = getCanaryPayload()
  const comment = renderCanaryComment(payload)

  return (
    <html lang="en">
      <body className="antialiased text-gray-900">
        {/* Canary comment - Beautiful Soup can find this in the HTML */}
        <div 
          aria-hidden 
          style={{ display: 'none' }}
          data-canary-comment
          dangerouslySetInnerHTML={{ __html: comment }}
        />
        {children}
        <AnalyticsWrapper />
      </body>
    </html>
  );
}
