import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Phantom — AI-Powered Screenshot Analysis for macOS",
  description:
    "Capture your screen with a shortcut, get instant AI responses powered by Google Gemini — all in a stealth floating overlay invisible to screen capture. Free and open source.",
  keywords:
    "screenshot analysis, AI assistant, macOS, stealth mode, Google Gemini, screen capture, Tauri, voice transcription, Whisper",
  authors: [{ name: "Enzo Spagnolli" }],
  robots: "index, follow",
  openGraph: {
    title: "Phantom — AI-Powered Screenshot Analysis for macOS",
    description:
      "Press a shortcut, get instant AI answers in a floating overlay invisible to screen capture.",
    type: "website",
    url: "https://enzo3322.github.io/phantom",
    siteName: "Phantom",
    locale: "en_US",
  },
  twitter: {
    card: "summary_large_image",
    title: "Phantom — AI-Powered Screenshot Analysis for macOS",
    description:
      "Press a shortcut, get instant AI answers in a floating overlay invisible to screen capture.",
  },
  other: {
    "theme-color": "#0a0a0a",
    "apple-mobile-web-app-capable": "yes",
    "apple-mobile-web-app-status-bar-style": "black-translucent",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <head>
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{
            __html: JSON.stringify({
              "@context": "https://schema.org",
              "@type": "SoftwareApplication",
              name: "Phantom",
              operatingSystem: "macOS",
              applicationCategory: "UtilitiesApplication",
              description:
                "AI-powered screenshot analysis tool for macOS with stealth mode. Capture your screen with a shortcut and get instant AI responses in a floating overlay invisible to screen capture.",
              url: "https://enzo3322.github.io/phantom",
              downloadUrl:
                "https://enzo3322.github.io/phantom/downloads/Phantom.dmg",
              softwareVersion: "0.1.0",
              offers: {
                "@type": "Offer",
                price: "0",
                priceCurrency: "USD",
              },
              author: {
                "@type": "Person",
                name: "Enzo Spagnolli",
              },
            }),
          }}
        />
      </head>
      <body>{children}</body>
    </html>
  );
}
