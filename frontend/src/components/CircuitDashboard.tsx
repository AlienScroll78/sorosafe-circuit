'use client';

import React, { useState, useEffect } from 'react';
import { isConnected, getPublicKey, signTransaction } from '@stellar/freighter-api';

interface VaultStatus {
  address: string;
  state: 'CLOSED' | 'OPEN';
  lastBalance: string;
  threshold: string;
}

export default function CircuitDashboard() {
  const [walletConnected, setWalletConnected] = useState<boolean>(false);
  const [publicKey, setPublicKey] = useState<string>('');
  const [vaultAddress, setVaultAddress] = useState<string>('');
  const [circuitStatus, setCircuitStatus] = useState<string>('CLOSED (MONITORING)');
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string>('');

  useEffect(() => {
    checkConnection();
  }, []);

  const checkConnection = async () => {
    try {
      if (await isConnected()) {
        const key = await getPublicKey();
        setPublicKey(key);
        setWalletConnected(true);
      }
    } catch (err) {
      console.error('Freighter not found', err);
    }
  };

  const connectWallet = async () => {
    try {
      const key = await getPublicKey();
      setPublicKey(key);
      setWalletConnected(true);
      setError('');
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Wallet connection rejected';
      console.error('Wallet connection error:', err);
      setError(errorMsg);
    }
  };

  const handleManualTrip = async () => {
    if (!vaultAddress) {
      setError('Please enter a valid vault address');
      return;
    }

    if (!walletConnected || !publicKey) {
      setError('Wallet not connected');
      return;
    }

    setLoading(true);
    setError('');

    try {
      // In production, this would:
      // 1. Build the XDR transaction for emergency_trip
      // 2. Sign with Freighter
      // 3. Submit to Soroban RPC endpoint
      console.log(`[SoroSafe] Emergency trip initiated for vault: ${vaultAddress} by ${publicKey}`);
      
      // Simulated delay for demo purposes
      await new Promise(resolve => setTimeout(resolve, 1500));
      
      setCircuitStatus('OPEN (TRIPPED)');
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : 'Failed to execute trip';
      console.error('Trip execution error:', e);
      setError(errorMsg);
    } finally {
      setLoading(false);
    }
  };

  const handleQueryState = async () => {
    if (!vaultAddress) {
      setError('Please enter a vault address');
      return;
    }

    setLoading(true);
    setError('');

    try {
      // In production, this would query verify_state from the contract
      console.log(`[SoroSafe] Querying state for vault: ${vaultAddress}`);
      
      await new Promise(resolve => setTimeout(resolve, 800));
      
      // Simulated response
      setCircuitStatus('CLOSED (MONITORING)');
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : 'Failed to query state';
      console.error('Query error:', e);
      setError(errorMsg);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{ display: 'grid', gap: '2rem' }}>
      {error && (
        <div
          style={{
            padding: '1rem',
            background: '#fee',
            border: '1px solid #fcc',
            borderRadius: '4px',
            color: '#c00',
            fontSize: '0.9rem',
          }}
        >
          {error}
        </div>
      )}

      <section
        style={{
          padding: '1.5rem',
          background: '#fff',
          borderRadius: '8px',
          boxShadow: '0 1px 3px rgba(0,0,0,0.1)',
        }}
      >
        <h2 style={{ fontSize: '1.25rem', marginTop: 0 }}>Warden Authentication</h2>
        {walletConnected && publicKey ? (
          <div>
            <span style={{ color: 'green', fontWeight: 600 }}>✓ Authenticated: </span>
            <code
              style={{
                background: '#eee',
                padding: '0.2rem 0.4rem',
                borderRadius: '4px',
                fontSize: '0.85rem',
              }}
            >
              {publicKey.slice(0, 10)}...{publicKey.slice(-10)}
            </code>
          </div>
        ) : (
          <button
            onClick={connectWallet}
            style={{
              padding: '0.75rem 1.5rem',
              background: '#111',
              color: '#fff',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              fontWeight: 600,
              fontSize: '0.95rem',
            }}
          >
            Connect Freighter Wallet
          </button>
        )}
      </section>

      <section
        style={{
          padding: '1.5rem',
          background: '#fff',
          borderRadius: '8px',
          boxShadow: '0 1px 3px rgba(0,0,0,0.1)',
        }}
      >
        <h2 style={{ fontSize: '1.25rem', marginTop: 0 }}>Vault Monitoring & Control</h2>

        <div style={{ marginBottom: '1.5rem' }}>
          <label style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}>
            Target Vault Address
          </label>
          <input
            type="text"
            placeholder="G..."
            value={vaultAddress}
            onChange={(e) => setVaultAddress(e.target.value)}
            style={{
              width: '100%',
              padding: '0.7rem',
              borderRadius: '4px',
              border: '1px solid #ccc',
              fontSize: '0.95rem',
              boxSizing: 'border-box',
            }}
          />
        </div>

        <div style={{ display: 'flex', gap: '1rem', marginBottom: '1.5rem', flexWrap: 'wrap' }}>
          <button
            onClick={handleQueryState}
            disabled={loading || !walletConnected}
            style={{
              padding: '0.75rem 1.5rem',
              background: '#0066cc',
              color: '#fff',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              fontWeight: 600,
              opacity: walletConnected ? 1 : 0.5,
              fontSize: '0.95rem',
            }}
          >
            {loading ? 'Querying...' : 'Query State'}
          </button>

          <button
            onClick={handleManualTrip}
            disabled={loading || !walletConnected}
            style={{
              padding: '0.75rem 1.5rem',
              background: '#e00000',
              color: '#fff',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              fontWeight: 600,
              opacity: walletConnected ? 1 : 0.5,
              fontSize: '0.95rem',
            }}
          >
            {loading ? 'Executing XDR...' : 'Force Emergency Trip'}
          </button>
        </div>

        <div
          style={{
            padding: '1rem',
            background: '#f9f9f9',
            borderRadius: '4px',
            border: '1px solid #eee',
          }}
        >
          <span style={{ fontWeight: 500 }}>Circuit State: </span>
          <strong
            style={{
              color: circuitStatus.includes('OPEN') ? '#e00000' : '#00aa00',
              fontSize: '1.05rem',
            }}
          >
            {circuitStatus}
          </strong>
        </div>
      </section>

      <section
        style={{
          padding: '1.5rem',
          background: '#fff',
          borderRadius: '8px',
          boxShadow: '0 1px 3px rgba(0,0,0,0.1)',
        }}
      >
        <h2 style={{ fontSize: '1.25rem', marginTop: 0 }}>Quick Reference</h2>
        <ul style={{ lineHeight: '1.8', color: '#555' }}>
          <li>
            <strong>Verify State:</strong> Check vault balance against threshold. Auto-trips if
            drain exceeds limit.
          </li>
          <li>
            <strong>Emergency Trip:</strong> Manual override by owner or authorized warden to
            immediately trip circuit.
          </li>
          <li>
            <strong>Evacuation:</strong> When tripped, all remaining vault assets transfer to
            emergency address.
          </li>
        </ul>
      </section>
    </div>
  );
}
