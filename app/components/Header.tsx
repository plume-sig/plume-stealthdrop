import React from "react";

const Header: React.FC = () => {
    return (
        <>
            <h1 className="text-4xl font-bold text-center mb-4 text-dark">
                stealthdrop
            </h1>
            <p className="text-xl text-center mb-8 text-gray-600">
                Anonymous Airdrops using ZK-SNARKs and PLUMEs
            </p>
        </>
    );
};

export default Header;
