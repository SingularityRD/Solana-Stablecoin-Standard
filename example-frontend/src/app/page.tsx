'use client';

import { useState } from 'react';

export default function Home() {
  const [connected, setConnected] = useState(false);
  const [supply, setSupply] = useState(1000000);
  const [paused, setPaused] = useState(false);

  return (
    <div className="min-h-screen bg-gradient-to-b from-blue-900 to-purple-900 text-white p-8">
      <div className="container mx-auto">
        <h1 className="text-4xl font-bold text-center mb-8">SSS Token Dashboard</h1>
        
        {!connected ? (
          <button
            onClick={() => setConnected(true)}
            className="bg-blue-500 hover:bg-blue-600 px-6 py-3 rounded-lg font-semibold"
          >
            Connect Wallet
          </button>
        ) : (
          <div className="space-y-6">
            <div className="bg-white/10 p-6 rounded-lg">
              <h2 className="text-2xl font-semibold mb-4">Dashboard</h2>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div className="bg-white/5 p-4 rounded">
                  <p className="text-sm text-gray-400">Total Supply</p>
                  <p className="text-2xl font-bold">{supply.toLocaleString()}</p>
                </div>
                <div className="bg-white/5 p-4 rounded">
                  <p className="text-sm text-gray-400">Status</p>
                  <p className="text-2xl font-bold">{paused ? 'Paused' : 'Active'}</p>
                </div>
                <div className="bg-white/5 p-4 rounded">
                  <p className="text-sm text-gray-400">Preset</p>
                  <p className="text-2xl font-bold">SSS-2</p>
                </div>
                <div className="bg-white/5 p-4 rounded">
                  <p className="text-sm text-gray-400">Blacklist</p>
                  <p className="text-2xl font-bold">3</p>
                </div>
              </div>
              
              <div className="mt-6 flex gap-4 flex-wrap">
                <button className="bg-blue-500 hover:bg-blue-600 px-4 py-2 rounded">Mint</button>
                <button className="bg-red-500 hover:bg-red-600 px-4 py-2 rounded">Burn</button>
                <button className="bg-yellow-500 hover:bg-yellow-600 px-4 py-2 rounded">Freeze</button>
                <button 
                  onClick={() => setPaused(!paused)}
                  className="bg-purple-500 hover:bg-purple-600 px-4 py-2 rounded"
                >
                  {paused ? 'Unpause' : 'Pause'}
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
