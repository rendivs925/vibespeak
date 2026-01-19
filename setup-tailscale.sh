#!/bin/bash
# Vibespeak Tailscale Setup Script
# This script helps set up Tailscale for remote access to Vibespeak

set -e

echo "🎤 Vibespeak Tailscale Setup"
echo "=============================="
echo ""

# Check if running as root or with sudo
if [[ $EUID -eq 0 ]]; then
   echo "❌ This script should not be run as root. Please run as a regular user with sudo access."
   exit 1
fi

# Check if Tailscale is installed
if ! command -v tailscale &> /dev/null; then
    echo "📦 Installing Tailscale..."

    # Install Tailscale
    curl -fsSL https://tailscale.com/install.sh | sh

    echo "✅ Tailscale installed successfully"
else
    echo "✅ Tailscale is already installed"
fi

# Start Tailscale service
echo "🔄 Starting Tailscale service..."
sudo systemctl enable --now tailscaled

# Check if already logged in
if tailscale status &> /dev/null; then
    echo "✅ Already logged into Tailscale"

    # Show current status
    echo ""
    echo "📊 Current Tailscale Status:"
    tailscale status
    echo ""
    echo "🌐 Your Tailscale IP:"
    tailscale ip -4
else
    echo "🔐 Logging into Tailscale..."
    echo "A browser window will open for authentication."
    echo "After authenticating, return here and press Enter to continue."
    echo ""

    # Open login (non-interactive)
    sudo tailscale login --web

    echo "Press Enter after completing authentication in your browser..."
    read -r

    # Verify login
    if tailscale status &> /dev/null; then
        echo "✅ Successfully authenticated with Tailscale"
    else
        echo "❌ Authentication failed. Please try again."
        exit 1
    fi
fi

# Get Tailscale IP
TAILSCALE_IP=$(tailscale ip -4 2>/dev/null || echo "")
if [ -n "$TAILSCALE_IP" ]; then
    echo ""
    echo "🎉 Setup Complete!"
    echo "=================="
    echo ""
    echo "📱 Access Vibespeak remotely at:"
    echo "   http://$TAILSCALE_IP:8080"
    echo ""
    echo "📋 Next steps:"
    echo "   1. Open Vibespeak web interface"
    echo "   2. Go to Settings > Tailscale Remote Access"
    echo "   3. Enable Tailscale and enter IP: $TAILSCALE_IP"
    echo "   4. Install Tailscale on your mobile device"
    echo "   5. Access Vibespeak from anywhere!"
    echo ""
    echo "📖 For detailed instructions, see: docs/TAILSCALE_SETUP.md"
else
    echo "⚠️  Could not determine Tailscale IP. Please run 'tailscale ip -4' manually."
fi

echo ""
echo "🔧 Useful Tailscale commands:"
echo "   tailscale status          # Check connection status"
echo "   tailscale ip -4           # Get your IP address"
echo "   tailscale logout          # Sign out"
echo "   sudo tailscale down       # Disconnect"
echo ""

# Check if Vibespeak config needs updating
if [ -f "config/system.json" ]; then
    echo "🔄 Updating Vibespeak configuration..."
    # This would be done through the web interface, but we can suggest it
    echo "   Please visit the web interface and enable Tailscale in Settings"
fi