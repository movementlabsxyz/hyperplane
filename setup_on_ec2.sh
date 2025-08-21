#!/bin/bash

set -e  # Exit immediately if any command fails

# echo "Updating system..."
# sudo yum update -y

# echo "Installing Git..."
# sudo yum install -y git

# echo "Downloading Hyperplane repo..."
# git clone --recursive https://github.com/movementlabsxyz/hyperplane.git
# cd hyperplane

echo "Installing Rust..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

echo "Make cargo available immediately"
source "$HOME/.cargo/env"

echo "Persist cargo path for all future sessions"
echo 'source $HOME/.cargo/env' >> ~/.bashrc

echo "Installing minimal C build tools (gcc, g++)"
sudo yum install -y gcc gcc-c++ make

echo "Installing Python 3 and pip"
sudo yum install -y python3 python3-pip

echo "Installing Python plotting libraries"
pip3 install matplotlib numpy pandas scipy "python-dateutil<=2.9.0"

echo "✅ Setup complete!"
echo "Next: run your simulation:"
echo "   ./simulator/run.sh"

echo ""
echo "💡 Tip: To keep the simulation running after disconnecting, use screen:"
echo "   sudo yum install -y screen"
echo "   screen -S sim_session"
echo "   ./simulator/run.sh"
echo ""
echo "   # Detach from screen (keep it running): Ctrl+A then D"
echo "   # Reattach later: screen -r sim_session"