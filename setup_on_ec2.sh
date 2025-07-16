#!/bin/bash

set -e  # Exit immediately if any command fails

# echo "Updating system..."
# sudo yum update -y

# echo "Installing Git..."
# sudo yum install -y git

# echo "Downloading Hyperplane repo..."
# git clone https://github.com/movementlabsxyz/hyperplane.git
# cd hyperplane

echo "Cloning Hyperplane repo with submodules..."
git clone --recursive https://github.com/movementlabsxyz/hyperplane.git

echo "Installing Rust..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

echo "Installing minimal C build tools (gcc, g++)..."
sudo yum install -y gcc gcc-c++ make

echo "Installing Python 3 and pip..."
sudo yum install -y python3 python3-pip

echo "Installing Python plotting libraries..."
pip3 install matplotlib numpy pandas scipy "python-dateutil<=2.9.0"

echo "✅ Setup complete!"
echo "Next: run your simulation:"
echo "   ./run_tests.sh 1 0"
