#!/bin/bash

# Exit on error
set -e

# Build the frontend
echo "Building frontend..."
cd frontend
npm install
npm run build
cd ..

# Build the backend
echo "Building backend..."
cd backend
cargo build --release
cd ..

# Create the distribution directory
echo "Creating distribution directory..."
mkdir -p dist/frontend
mkdir -p dist/output

# Copy the files
echo "Copying files..."
cp backend/target/release/backend dist/
cp -r frontend/build/* dist/frontend/

echo "Build complete."
