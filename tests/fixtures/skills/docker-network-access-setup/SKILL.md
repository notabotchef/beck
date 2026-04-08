---
name: docker-network-access-setup
description: Set up Docker services for local network access from other devices (iPad, mobile, etc.) on same WiFi
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [Docker, Network, Mobile-Access, iPad, WiFi, Local-Network]
    related_skills: []
---

# Docker Network Access Setup

Enable access to Docker services from other devices on the same local network (iPad, phones, etc.).

## When to Use

- User wants to access web app from iPad/mobile on same WiFi
- Docker services running locally but need network access
- Development setup needs multi-device testing
- Local Docker stack needs to be accessible network-wide

## Prerequisites

- Docker Compose setup with exposed ports
- Devices on same WiFi network
- Basic understanding of local network addressing

## Workflow

### 1. Check Current Docker Status

```bash
cd <project_directory>
docker compose ps
```

Verify services are running and ports are exposed.

### 2. Find Host IP Address

**macOS:**
```bash
ifconfig en0 | grep "inet " | awk '{print $2}'
```

**Linux:**
```bash
ip route get 1 | awk '{print $7; exit}'
# or
hostname -I | awk '{print $1}'
```

**Windows:**
```bash
ipconfig | findstr "IPv4"
```

### 3. Verify Docker Port Mapping

Look for port mappings in `docker-compose.yml`:
```yaml
services:
  nginx:
    ports:
      - "8080:80"  # Host:Container
```

This maps host port 8080 to container port 80.

### 4. Test Local Access First

```bash
curl -I http://localhost:8080
```

Should return HTTP 200 OK response.

### 5. Build Network URL

Format: `http://<HOST_IP>:<HOST_PORT>`

Example: `http://10.0.0.39:8080`

### 6. Test Network Access

From host machine:
```bash
curl -I http://<HOST_IP>:<HOST_PORT>
```

### 7. Troubleshoot Firewall (if needed)

**macOS Application Firewall:**
```bash
# Check status
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate

# Allow Docker/nginx if blocked
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --add /usr/bin/nginx
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --unblock /usr/bin/nginx
```

**Linux (ufw):**
```bash
# Allow specific port
sudo ufw allow 8080

# Or allow from specific subnet
sudo ufw allow from 192.168.1.0/24 to any port 8080
```

### 8. Device Access Instructions

**For user:**
1. Open browser on iPad/device
2. Navigate to: `http://<HOST_IP>:<HOST_PORT>`
3. Bookmark for easy access

## Common Port Patterns

| Service | Typical Port | Example URL |
|---------|--------------|-------------|
| Nginx Proxy | 8080 | http://10.0.0.39:8080 |
| Development Server | 3000 | http://10.0.0.39:3000 |
| API Backend | 5000-5050 | http://10.0.0.39:5050 |
| Database (dev) | 5432, 3306 | Direct connection |

## Docker Compose Best Practices

### Expose Ports Correctly
```yaml
services:
  web:
    ports:
      - "8080:80"  # ✅ Host:Container
    # NOT expose: ["80"]  # ❌ Internal only
```

### Use Nginx Proxy for Multiple Services
```yaml
nginx:
  image: nginx:alpine
  ports:
    - "8080:80"
  volumes:
    - ./nginx.conf:/etc/nginx/conf.d/default.conf:ro
```

## Troubleshooting

### Connection Refused
1. Verify Docker service is running: `docker compose ps`
2. Check port mapping: `docker compose port <service> <port>`
3. Test localhost first: `curl localhost:<port>`

### Firewall Blocking
1. Check firewall status
2. Allow specific ports or applications
3. Test from another device on network

### Wrong IP Address
1. Verify network interface: WiFi vs Ethernet
2. Check for VPN interference
3. Use `ifconfig` / `ip addr` to confirm

### Device Can't Connect
1. Ensure same WiFi network
2. Check router AP isolation settings
3. Try different browser/clear cache
4. Verify URL format: `http://` not `https://`

## Security Considerations

- Only expose necessary ports
- Use authentication if sensitive data
- Consider network-level restrictions
- Monitor access logs
- Use HTTPS for production

## Example Complete Setup

```yaml
# docker-compose.yml
services:
  postgres:
    image: postgres:16-alpine
    ports:
      - "5432:5432"
    environment:
      POSTGRES_USER: app
      POSTGRES_PASSWORD: app

  backend:
    build: ./backend
    ports:
      - "5050:80"
    depends_on:
      - postgres

  frontend:
    build: ./frontend
    depends_on:
      - backend

  nginx:
    image: nginx:alpine
    ports:
      - "8080:80"  # Main access point
    volumes:
      - ./nginx.conf:/etc/nginx/conf.d/default.conf:ro
    depends_on:
      - frontend
      - backend
```

**Access URLs:**
- Main app: `http://<HOST_IP>:8080`
- API direct: `http://<HOST_IP>:5050`
- Database: `<HOST_IP>:5432`