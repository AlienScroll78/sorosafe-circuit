import React from 'react';

export const metadata = {
  title: 'SoroSafe Circuit Control',
  description: 'Institutional Vault Circuit Breaker',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body style={{ margin: 0, background: '#f5f5f5' }}>
        {children}
      </body>
    </html>
  );
}
