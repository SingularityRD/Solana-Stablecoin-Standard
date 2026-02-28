import type { Metadata } from 'next';
import { WalletProvider } from '../components/WalletProvider';
import { SDKProvider } from '../components/SDKProvider';
import './globals.css';

export const metadata: Metadata = {
  title: 'SSS Token Dashboard',
  description: 'Example frontend for Solana Stablecoin Standard',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body className="bg-gradient-to-b from-slate-900 via-purple-900 to-slate-900 min-h-screen text-white">
        <WalletProvider>
          <SDKProvider>
            {children}
          </SDKProvider>
        </WalletProvider>
      </body>
    </html>
  );
}