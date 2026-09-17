import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  devIndicators: false,
  reactStrictMode: true,
  transpilePackages: [
    "@contextlab/design-system",
    "@contextlab/local-sdk",
    "@contextlab/ts-sdk",
    "@contextlab/ui"
  ]
};

export default nextConfig;
