'use client';

import { FC, useState } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import { useSDK } from './SDKProvider';

interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  children: React.ReactNode;
}

const Modal: FC<ModalProps> = ({ isOpen, onClose, title, children }) => {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-xl p-6 max-w-md w-full mx-4 border border-gray-700">
        <div className="flex justify-between items-center mb-4">
          <h3 className="text-xl font-semibold">{title}</h3>
          <button onClick={onClose} className="text-gray-400 hover:text-white">
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        {children}
      </div>
    </div>
  );
};

interface ToastProps {
  message: string;
  type: 'success' | 'error' | 'info';
  onClose: () => void;
}

const Toast: FC<ToastProps> = ({ message, type, onClose }) => {
  const bgColor = type === 'success' ? 'bg-green-600' : type === 'error' ? 'bg-red-600' : 'bg-blue-600';
  
  return (
    <div className={`fixed bottom-4 right-4 ${bgColor} text-white px-6 py-3 rounded-lg shadow-lg flex items-center gap-3 z-50`}>
      <span>{message}</span>
      <button onClick={onClose} className="text-white/80 hover:text-white">
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>
  );
};

export const Dashboard: FC = () => {
  const { connected, publicKey, disconnect } = useWallet();
  const sdk = useSDK();
  
  const [modalType, setModalType] = useState<'mint' | 'burn' | 'freeze' | 'thaw' | 'seize' | null>(null);
  const [toast, setToast] = useState<{ message: string; type: 'success' | 'error' | 'info' } | null>(null);
  
  // Form states
  const [mintAmount, setMintAmount] = useState('');
  const [mintRecipient, setMintRecipient] = useState('');
  const [burnAmount, setBurnAmount] = useState('');
  const [burnAccount, setBurnAccount] = useState('');
  const [freezeAccount, setFreezeAccount] = useState('');
  const [thawAccount, setThawAccount] = useState('');
  const [seizeFrom, setSeizeFrom] = useState('');
  const [seizeTo, setSeizeTo] = useState('');
  const [seizeAmount, setSeizeAmount] = useState('');

  const showToast = (message: string, type: 'success' | 'error' | 'info') => {
    setToast({ message, type });
    setTimeout(() => setToast(null), 4000);
  };

  const handleMint = async () => {
    if (!mintAmount || !mintRecipient) {
      showToast('Please fill in all fields', 'error');
      return;
    }
    
    try {
      const signature = await sdk.mint(mintRecipient, parseInt(mintAmount));
      showToast(`Minted ${mintAmount} tokens! Signature: ${signature.slice(0, 20)}...`, 'success');
      setModalType(null);
      setMintAmount('');
      setMintRecipient('');
    } catch (err) {
      showToast(err instanceof Error ? err.message : 'Mint failed', 'error');
    }
  };

  const handleBurn = async () => {
    if (!burnAmount || !burnAccount) {
      showToast('Please fill in all fields', 'error');
      return;
    }
    
    try {
      const signature = await sdk.burn(burnAccount, parseInt(burnAmount));
      showToast(`Burned ${burnAmount} tokens! Signature: ${signature.slice(0, 20)}...`, 'success');
      setModalType(null);
      setBurnAmount('');
      setBurnAccount('');
    } catch (err) {
      showToast(err instanceof Error ? err.message : 'Burn failed', 'error');
    }
  };

  const handleFreeze = async () => {
    if (!freezeAccount) {
      showToast('Please enter an account address', 'error');
      return;
    }
    
    try {
      const signature = await sdk.freeze(freezeAccount);
      showToast(`Account frozen! Signature: ${signature.slice(0, 20)}...`, 'success');
      setModalType(null);
      setFreezeAccount('');
    } catch (err) {
      showToast(err instanceof Error ? err.message : 'Freeze failed', 'error');
    }
  };

  const handleThaw = async () => {
    if (!thawAccount) {
      showToast('Please enter an account address', 'error');
      return;
    }
    
    try {
      const signature = await sdk.thaw(thawAccount);
      showToast(`Account thawed! Signature: ${signature.slice(0, 20)}...`, 'success');
      setModalType(null);
      setThawAccount('');
    } catch (err) {
      showToast(err instanceof Error ? err.message : 'Thaw failed', 'error');
    }
  };

  const handleSeize = async () => {
    if (!seizeFrom || !seizeTo || !seizeAmount) {
      showToast('Please fill in all fields', 'error');
      return;
    }
    
    try {
      const signature = await sdk.seize(seizeFrom, seizeTo, parseInt(seizeAmount));
      showToast(`Seized ${seizeAmount} tokens! Signature: ${signature.slice(0, 20)}...`, 'success');
      setModalType(null);
      setSeizeFrom('');
      setSeizeTo('');
      setSeizeAmount('');
    } catch (err) {
      showToast(err instanceof Error ? err.message : 'Seize failed', 'error');
    }
  };

  const handlePause = async () => {
    try {
      if (sdk.stablecoinState?.paused) {
        const signature = await sdk.unpause();
        showToast(`Contract unpaused! Signature: ${signature.slice(0, 20)}...`, 'success');
      } else {
        const signature = await sdk.pause();
        showToast(`Contract paused! Signature: ${signature.slice(0, 20)}...`, 'success');
      }
    } catch (err) {
      showToast(err instanceof Error ? err.message : 'Operation failed', 'error');
    }
  };

  if (!connected) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[60vh]">
        <div className="bg-white/10 backdrop-blur-lg rounded-2xl p-8 text-center max-w-md">
          <div className="w-20 h-20 mx-auto mb-6 bg-gradient-to-br from-solana-purple to-solana-teal rounded-full flex items-center justify-center">
            <svg className="w-10 h-10 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
            </svg>
          </div>
          <h2 className="text-2xl font-bold mb-2">Welcome to SSS Token</h2>
          <p className="text-gray-400 mb-6">Connect your wallet to access the dashboard</p>
          <WalletMultiButton className="!bg-gradient-to-r !from-solana-purple !to-solana-teal !rounded-xl !px-8 !py-3 !font-semibold !text-white hover:!opacity-90 !transition-opacity" />
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        <div>
          <h2 className="text-2xl font-bold">Dashboard</h2>
          <p className="text-gray-400 text-sm mt-1">
            Connected: {publicKey?.toBase58().slice(0, 8)}...{publicKey?.toBase58().slice(-8)}
          </p>
        </div>
        <div className="flex gap-3">
          <button
            onClick={() => sdk.refreshState()}
            disabled={sdk.loading}
            className="bg-gray-700 hover:bg-gray-600 px-4 py-2 rounded-lg transition-colors disabled:opacity-50"
          >
            {sdk.loading ? 'Loading...' : 'Refresh'}
          </button>
          <WalletMultiButton className="!bg-gray-700 !rounded-lg !px-4 !py-2 hover:!bg-gray-600" />
        </div>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="bg-white/5 backdrop-blur rounded-xl p-5 border border-white/10">
          <div className="flex items-center justify-between">
            <p className="text-sm text-gray-400">Total Supply</p>
            <div className="w-10 h-10 bg-blue-500/20 rounded-lg flex items-center justify-center">
              <svg className="w-5 h-5 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
          </div>
          <p className="text-2xl font-bold mt-2">
            {sdk.stablecoinState ? sdk.stablecoinState.totalSupply.toLocaleString() : '---'}
          </p>
        </div>

        <div className="bg-white/5 backdrop-blur rounded-xl p-5 border border-white/10">
          <div className="flex items-center justify-between">
            <p className="text-sm text-gray-400">Status</p>
            <div className={`w-10 h-10 rounded-lg flex items-center justify-center ${sdk.stablecoinState?.paused ? 'bg-red-500/20' : 'bg-green-500/20'}`}>
              <div className={`w-3 h-3 rounded-full ${sdk.stablecoinState?.paused ? 'bg-red-400' : 'bg-green-400'}`} />
            </div>
          </div>
          <p className={`text-2xl font-bold mt-2 ${sdk.stablecoinState?.paused ? 'text-red-400' : 'text-green-400'}`}>
            {sdk.stablecoinState?.paused ? 'Paused' : 'Active'}
          </p>
        </div>

        <div className="bg-white/5 backdrop-blur rounded-xl p-5 border border-white/10">
          <div className="flex items-center justify-between">
            <p className="text-sm text-gray-400">Preset</p>
            <div className="w-10 h-10 bg-purple-500/20 rounded-lg flex items-center justify-center">
              <span className="text-purple-400 font-bold text-sm">SSS</span>
            </div>
          </div>
          <p className="text-2xl font-bold mt-2">
            SSS-{sdk.stablecoinState?.preset || '---'}
          </p>
        </div>

        <div className="bg-white/5 backdrop-blur rounded-xl p-5 border border-white/10">
          <div className="flex items-center justify-between">
            <p className="text-sm text-gray-400">Blacklist</p>
            <div className="w-10 h-10 bg-orange-500/20 rounded-lg flex items-center justify-center">
              <svg className="w-5 h-5 text-orange-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
              </svg>
            </div>
          </div>
          <p className="text-2xl font-bold mt-2">{sdk.getBlacklistCount()}</p>
        </div>
      </div>

      {/* Action Buttons */}
      <div className="bg-white/5 backdrop-blur rounded-xl p-6 border border-white/10">
        <h3 className="text-lg font-semibold mb-4">Operations</h3>
        <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-3">
          <button
            onClick={() => setModalType('mint')}
            disabled={sdk.loading || sdk.stablecoinState?.paused}
            className="bg-blue-600 hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed px-4 py-3 rounded-lg font-medium transition-colors flex items-center justify-center gap-2"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
            </svg>
            Mint
          </button>

          <button
            onClick={() => setModalType('burn')}
            disabled={sdk.loading || sdk.stablecoinState?.paused}
            className="bg-red-600 hover:bg-red-700 disabled:opacity-50 disabled:cursor-not-allowed px-4 py-3 rounded-lg font-medium transition-colors flex items-center justify-center gap-2"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17.657 18.657A8 8 0 016.343 7.343S7 9 9 10c0-2 .5-5 2.986-7C14 5 16.09 5.777 17.656 7.343A7.975 7.975 0 0120 13a7.975 7.975 0 01-2.343 5.657z" />
            </svg>
            Burn
          </button>

          <button
            onClick={() => setModalType('freeze')}
            disabled={sdk.loading}
            className="bg-yellow-600 hover:bg-yellow-700 disabled:opacity-50 disabled:cursor-not-allowed px-4 py-3 rounded-lg font-medium transition-colors flex items-center justify-center gap-2"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
            </svg>
            Freeze
          </button>

          <button
            onClick={() => setModalType('thaw')}
            disabled={sdk.loading}
            className="bg-green-600 hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed px-4 py-3 rounded-lg font-medium transition-colors flex items-center justify-center gap-2"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 11V7a4 4 0 118 0m-4 8v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2z" />
            </svg>
            Thaw
          </button>

          <button
            onClick={() => setModalType('seize')}
            disabled={sdk.loading}
            className="bg-orange-600 hover:bg-orange-700 disabled:opacity-50 disabled:cursor-not-allowed px-4 py-3 rounded-lg font-medium transition-colors flex items-center justify-center gap-2"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4" />
            </svg>
            Seize
          </button>

          <button
            onClick={handlePause}
            disabled={sdk.loading}
            className={`${sdk.stablecoinState?.paused ? 'bg-green-600 hover:bg-green-700' : 'bg-purple-600 hover:bg-purple-700'} disabled:opacity-50 disabled:cursor-not-allowed px-4 py-3 rounded-lg font-medium transition-colors flex items-center justify-center gap-2`}
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d={sdk.stablecoinState?.paused ? "M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" : "M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z"} />
            </svg>
            {sdk.stablecoinState?.paused ? 'Unpause' : 'Pause'}
          </button>
        </div>
      </div>

      {/* Compliance Info */}
      <div className="bg-white/5 backdrop-blur rounded-xl p-6 border border-white/10">
        <h3 className="text-lg font-semibold mb-4">Compliance</h3>
        <div className="flex items-center gap-3">
          <div className={`w-3 h-3 rounded-full ${sdk.stablecoinState?.complianceEnabled ? 'bg-green-400' : 'bg-gray-400'}`} />
          <span className="text-gray-300">
            Compliance {sdk.stablecoinState?.complianceEnabled ? 'Enabled' : 'Disabled'}
          </span>
        </div>
        <p className="text-gray-500 text-sm mt-2">
          SSS-2 preset includes blacklist, freeze, and seize capabilities for regulatory compliance.
        </p>
      </div>

      {/* Modals */}
      <Modal isOpen={modalType === 'mint'} onClose={() => setModalType(null)} title="Mint Tokens">
        <div className="space-y-4">
          <div>
            <label className="block text-sm text-gray-400 mb-1">Recipient Address</label>
            <input
              type="text"
              value={mintRecipient}
              onChange={(e) => setMintRecipient(e.target.value)}
              placeholder="Enter wallet address"
              className="w-full bg-gray-700 rounded-lg px-4 py-2 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-solana-purple"
            />
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-1">Amount</label>
            <input
              type="number"
              value={mintAmount}
              onChange={(e) => setMintAmount(e.target.value)}
              placeholder="Enter amount"
              className="w-full bg-gray-700 rounded-lg px-4 py-2 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-solana-purple"
            />
          </div>
          <button
            onClick={handleMint}
            disabled={sdk.loading}
            className="w-full bg-gradient-to-r from-solana-purple to-solana-teal py-2 rounded-lg font-medium hover:opacity-90 transition-opacity disabled:opacity-50"
          >
            {sdk.loading ? 'Processing...' : 'Mint Tokens'}
          </button>
        </div>
      </Modal>

      <Modal isOpen={modalType === 'burn'} onClose={() => setModalType(null)} title="Burn Tokens">
        <div className="space-y-4">
          <div>
            <label className="block text-sm text-gray-400 mb-1">Account Address</label>
            <input
              type="text"
              value={burnAccount}
              onChange={(e) => setBurnAccount(e.target.value)}
              placeholder="Enter token account address"
              className="w-full bg-gray-700 rounded-lg px-4 py-2 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-solana-purple"
            />
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-1">Amount</label>
            <input
              type="number"
              value={burnAmount}
              onChange={(e) => setBurnAmount(e.target.value)}
              placeholder="Enter amount"
              className="w-full bg-gray-700 rounded-lg px-4 py-2 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-solana-purple"
            />
          </div>
          <button
            onClick={handleBurn}
            disabled={sdk.loading}
            className="w-full bg-red-600 hover:bg-red-700 py-2 rounded-lg font-medium transition-colors disabled:opacity-50"
          >
            {sdk.loading ? 'Processing...' : 'Burn Tokens'}
          </button>
        </div>
      </Modal>

      <Modal isOpen={modalType === 'freeze'} onClose={() => setModalType(null)} title="Freeze Account">
        <div className="space-y-4">
          <div>
            <label className="block text-sm text-gray-400 mb-1">Account Address</label>
            <input
              type="text"
              value={freezeAccount}
              onChange={(e) => setFreezeAccount(e.target.value)}
              placeholder="Enter token account address"
              className="w-full bg-gray-700 rounded-lg px-4 py-2 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-solana-purple"
            />
          </div>
          <p className="text-yellow-400 text-sm">
            Warning: This will freeze the account and prevent all transfers.
          </p>
          <button
            onClick={handleFreeze}
            disabled={sdk.loading}
            className="w-full bg-yellow-600 hover:bg-yellow-700 py-2 rounded-lg font-medium transition-colors disabled:opacity-50"
          >
            {sdk.loading ? 'Processing...' : 'Freeze Account'}
          </button>
        </div>
      </Modal>

      <Modal isOpen={modalType === 'thaw'} onClose={() => setModalType(null)} title="Thaw Account">
        <div className="space-y-4">
          <div>
            <label className="block text-sm text-gray-400 mb-1">Account Address</label>
            <input
              type="text"
              value={thawAccount}
              onChange={(e) => setThawAccount(e.target.value)}
              placeholder="Enter token account address"
              className="w-full bg-gray-700 rounded-lg px-4 py-2 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-solana-purple"
            />
          </div>
          <button
            onClick={handleThaw}
            disabled={sdk.loading}
            className="w-full bg-green-600 hover:bg-green-700 py-2 rounded-lg font-medium transition-colors disabled:opacity-50"
          >
            {sdk.loading ? 'Processing...' : 'Thaw Account'}
          </button>
        </div>
      </Modal>

      <Modal isOpen={modalType === 'seize'} onClose={() => setModalType(null)} title="Seize Tokens">
        <div className="space-y-4">
          <div>
            <label className="block text-sm text-gray-400 mb-1">From Account</label>
            <input
              type="text"
              value={seizeFrom}
              onChange={(e) => setSeizeFrom(e.target.value)}
              placeholder="Enter source account address"
              className="w-full bg-gray-700 rounded-lg px-4 py-2 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-solana-purple"
            />
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-1">To Account</label>
            <input
              type="text"
              value={seizeTo}
              onChange={(e) => setSeizeTo(e.target.value)}
              placeholder="Enter destination account address"
              className="w-full bg-gray-700 rounded-lg px-4 py-2 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-solana-purple"
            />
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-1">Amount</label>
            <input
              type="number"
              value={seizeAmount}
              onChange={(e) => setSeizeAmount(e.target.value)}
              placeholder="Enter amount"
              className="w-full bg-gray-700 rounded-lg px-4 py-2 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-solana-purple"
            />
          </div>
          <p className="text-orange-400 text-sm">
            Warning: This will transfer tokens without owner consent.
          </p>
          <button
            onClick={handleSeize}
            disabled={sdk.loading}
            className="w-full bg-orange-600 hover:bg-orange-700 py-2 rounded-lg font-medium transition-colors disabled:opacity-50"
          >
            {sdk.loading ? 'Processing...' : 'Seize Tokens'}
          </button>
        </div>
      </Modal>

      {/* Toast */}
      {toast && <Toast message={toast.message} type={toast.type} onClose={() => setToast(null)} />}
    </div>
  );
};
