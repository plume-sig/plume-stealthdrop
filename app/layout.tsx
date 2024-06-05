import Footer from "./components/Footer";
import "./globals.css";

export default function RootLayout({
    children,
}: {
    children: React.ReactNode;
}) {
    return (
        <html lang="en">
            <body className="flex flex-col min-h-screen bg-gray-900">
                {children}
                <Footer />
            </body>
        </html>
    );
}
