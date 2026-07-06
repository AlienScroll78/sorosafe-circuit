import CircuitDashboard from '@/components/CircuitDashboard';

export default function Home() {
  return (
    <main
      style={{
        padding: '2rem',
        fontFamily: 'system-ui, sans-serif',
        maxWidth: '1000px',
        margin: '0 auto',
      }}
    >
      <header
        style={{
          borderBottom: '1px solid #eaeaea',
          paddingBottom: '1rem',
          marginBottom: '2rem',
        }}
      >
        <h1 style={{ fontSize: '2rem', fontWeight: 700, margin: 0 }}>
          SoroSafe Guardian
        </h1>
        <p style={{ color: '#666', marginTop: '0.5rem' }}>
          Zero-Trust Decentralized Circuit Breaker
        </p>
      </header>
      <CircuitDashboard />
    </main>
  );
}
