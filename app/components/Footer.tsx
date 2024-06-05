import React from "react";

const Footer: React.FC = () => {
    return (
        <footer className="bg-gray-800 text-white py-4">
            <div className="container mx-auto px-4 flex justify-between items-center">
                <p className="text-sm">
                    &copy; {new Date().getFullYear()} StealthDrop using PLUME
                </p>
                <a
                    href="https://github.com/plume-sig/plume-stealthdrop"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-white hover:text-gray-300"
                >
                    <i className="fab fa-github fa-lg"></i> GitHub
                </a>
            </div>
        </footer>
    );
};

export default Footer;
