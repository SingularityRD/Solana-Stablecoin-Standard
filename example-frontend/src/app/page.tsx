'use client';

import { Dashboard } from '../components/Dashboard';

export default function Home() {
  return (
    <main className="min-h-screen">
      <div className="container mx-auto px-4 py-8">
        <header className="text-center mb-12">
          <div className="inline-flex items-center gap-3 mb-4">
            <div className="w-12 h-12 bg-gradient-to-br from-solana-purple to-solana-teal rounded-xl flex items-center justify-center">
              <svg className="w-7 h-7 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <h1 className="text-4xl font-bold bg-gradient-to-r from-solana-purple to-solana-teal bg-clip-text text-transparent">
              SSS Token
            </h1>
          </div>
          <p className="text-gray-400 max-w-2xl mx-auto">
            Solana Stablecoin Standard - A compliant, programmable stablecoin framework with 
            built-in regulatory controls including mint, burn, freeze, seize, and pause capabilities.
          </p>
        </header>
        
        <Dashboard />
        
        <footer className="mt-16 text-center text-gray-500 text-sm">
          <p>Built on Solana using Anchor Framework</p>
          <p className="mt-2">
            <a href="https://github.com/solana-labs" className="text-solana-teal hover:underline">
              Documentation
            </a>
            {' | '}
            <a href="#" className="text-solana-purple hover:underline">
              GitHub
            </a>
          </p>
        </footer>
      </div>
    </main>
  );
}