# Tailscale Integration for Vibespeak

Tailscale provides secure, zero-configuration remote access to Vibespeak through a mesh VPN. This allows you to access your voice automation system from anywhere without exposing ports to the internet.

## Overview

Tailscale creates a secure network between your devices using WireGuard. Once set up, Vibespeak can be accessed from any device on your Tailscale network using private IP addresses.

## Prerequisites

- **Tailscale Account**: Free account at [tailscale.com](https://tailscale.com)
- **Devices**: Install Tailscale on your Vibespeak server and client devices
- **Network Access**: Ability to run Tailscale on your network

## Installation

### On Linux (Vibespeak Server)
```bash
# Install Tailscale
curl -fsSL https://tailscale.com/install.sh | sh

# Start Tailscale
sudo systemctl enable --now tailscaled

# Authenticate (follow the URL in terminal)
sudo tailscale up
```

### On Mobile Devices
- **iOS**: Download from App Store
- **Android**: Download from Google Play Store
- **Desktop**: Download from [tailscale.com](https://tailscale.com/download)

## Configuration

### 1. Enable Tailscale in Vibespeak

Edit your `config/system.json`:

```json
{
  "settings": {
    "tailscale_enabled": true,
    "web_server_bind": "100.64.0.1:8080",
    "tailscale_hostname": "vibespeak"
  }
}
```

Or use the web interface:
1. Go to Settings tab
2. Enable "Tailscale Remote Access"
3. Set bind address to your Tailscale IP
4. Configure hostname (optional)

### 2. Get Tailscale IP Address

```bash
# Check your Tailscale IP
tailscale ip -4
# Example output: 100.64.0.1

# Or check all IPs
ip addr show tailscale0
```

### 3. Configure Firewall (if needed)

Ensure Tailscale traffic can reach your Vibespeak port:

```bash
# Allow Tailscale interface traffic
sudo ufw allow in on tailscale0

# Or if using firewalld
sudo firewall-cmd --zone=trusted --add-interface=tailscale0
```

## Usage

### Accessing Vibespeak Remotely

1. **Connect Device to Tailscale**: Install Tailscale and log in with the same account
2. **Get Server IP**: Note the Tailscale IP from your server (`tailscale ip -4`)
3. **Access Vibespeak**: Open `http://[TAILSCALE_IP]:8080` in your browser

### Device Management

```bash
# List connected devices
tailscale status

# Ping other devices
tailscale ping [device-name]

# Get device IPs
tailscale ip -4 [device-name]
```

## Security Features

### Zero Trust Networking
- **Device Authorization**: Each device must be explicitly authorized
- **End-to-End Encryption**: All traffic encrypted with WireGuard
- **No Port Exposure**: No internet-exposed ports required
- **Network Segmentation**: Isolated from your regular network

### Vibespeak-Specific Security
- **Access Control**: Restrict which devices can access Vibespeak
- **Session Management**: Automatic session cleanup
- **Audit Logging**: Track remote access attempts

## Advanced Configuration

### Custom Hostname
Set a memorable name for your Vibespeak server:

```bash
sudo tailscale up --hostname=vibespeak-server
```

### Subnet Routing
Share your local network with Tailscale devices:

```bash
# Allow access to local subnet (be careful!)
sudo tailscale up --advertise-routes=192.168.1.0/24

# Approve routes in admin console
```

### ACL Policies
Create granular access policies in the Tailscale admin console:

```json
{
  "acls": [
    {
      "action": "accept",
      "src": ["autogroup:member"],
      "dst": ["vibespeak-server:8080"]
    }
  ]
}
```

## Troubleshooting

### Connection Issues

**Can't reach Vibespeak server:**
```bash
# Check Tailscale status
tailscale status

# Ping the server
tailscale ping vibespeak-server

# Check if service is running
sudo systemctl status tailscaled
```

**Port not accessible:**
```bash
# Check if Vibespeak is bound to correct interface
netstat -tlnp | grep 8080

# Verify Tailscale IP configuration
tailscale ip -4
```

**Firewall blocking:**
```bash
# Check firewall rules
sudo ufw status
sudo firewall-cmd --list-all
```

### Performance Issues

**High latency:**
- Check Tailscale relay usage: `tailscale ping --tsmp`
- Consider direct connections: `tailscale up --direct`

**Slow initial connection:**
- Enable MagicDNS: `tailscale up --accept-dns`
- Use shorter hostnames

### Common Errors

**"Tailscale not logged in":**
```bash
sudo tailscale login
```

**"Device not authorized":**
- Go to Tailscale admin console
- Approve the device
- Run `tailscale up` again

**"Permission denied":**
- Check file permissions for Tailscale config
- Ensure user is in correct group

## Mobile-Specific Setup

### iOS Setup
1. Install Tailscale from App Store
2. Sign in with your account
3. Enable "VPN" when prompted
4. Access Vibespeak using Tailscale IP

### Android Setup
1. Install Tailscale from Play Store
2. Sign in and authorize device
3. Enable "Always-on VPN" in settings
4. Use Tailscale IP to access Vibespeak

## Integration with Vibespeak Features

### Remote Voice Control
- Voice commands work seamlessly over Tailscale
- No quality degradation due to encryption
- Secure audio streaming

### Screen Sharing
- WebRTC screen sharing works through Tailscale
- No additional port forwarding required
- Encrypted peer-to-peer connections

### Command Execution
- All remote commands execute securely
- Audit logs available for security monitoring
- Granular permission control possible

## Monitoring and Maintenance

### Health Checks
```bash
# Check Tailscale health
tailscale ping --verbose [server]

# Monitor network usage
tailscale netcheck

# View logs
sudo journalctl -u tailscaled
```

### Updates
```bash
# Update Tailscale
sudo apt update && sudo apt install tailscale

# Restart service
sudo systemctl restart tailscaled
```

### Backup and Recovery
- Tailscale configuration is cloud-managed
- No local configuration to backup
- Device re-authorization required if reinstalled

## Best Practices

### Security
1. **Regular Audits**: Review connected devices monthly
2. **Principle of Least Privilege**: Only authorize necessary devices
3. **Network Segmentation**: Use ACLs to limit access
4. **Update Regularly**: Keep Tailscale and Vibespeak updated

### Performance
1. **Direct Connections**: Prefer direct device-to-device connections
2. **DNS Optimization**: Use MagicDNS for faster resolution
3. **Network Planning**: Consider device locations for optimal routing

### Management
1. **Naming Convention**: Use consistent device naming
2. **Documentation**: Keep inventory of authorized devices
3. **Monitoring**: Set up alerts for unusual access patterns

## Support and Resources

- **Tailscale Documentation**: [tailscale.com/kb](https://tailscale.com/kb)
- **Vibespeak Issues**: [GitHub Issues](https://github.com/rendivs925/vibespeak/issues)
- **Community Support**: [Tailscale Slack](https://tailscale.com/slack)

## Alternative Remote Access

If Tailscale doesn't meet your needs, consider:

- **WireGuard**: Self-hosted VPN for maximum control
- **ZeroTier**: Alternative mesh networking solution
- **ngrok**: Quick tunneling for development (not recommended for production)
- **Cloudflare Tunnel**: Secure tunneling with additional features

Tailscale provides the best balance of security, ease of use, and functionality for remote Vibespeak access.