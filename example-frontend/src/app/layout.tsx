export const metadata = {
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
      <body>{children}</body>
    </html>
  );
}
