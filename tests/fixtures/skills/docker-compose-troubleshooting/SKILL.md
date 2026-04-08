---
name: docker-compose-troubleshooting
description: Systematic approach for diagnosing and fixing Docker Compose issues - missing JS files, API routing problems, frontend-backend connectivity, and service startup failures
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [Docker, Docker-Compose, Debugging, Next.js, API-Proxy, Frontend-Backend]
    related_skills: [systematic-debugging, docker-network-access-setup]
---

# Docker Compose Troubleshooting

Systematic approach for diagnosing and fixing common Docker Compose issues, especially frontend-backend connectivity problems.

## Common Issue Patterns

### 1. Frontend Loading Issues
**Symptoms:**
- Website loads but nothing works properly
- Missing JavaScript files (404 errors in browser console)
- Side menu/UI components not loading
- Database not reading

**Diagnostic Steps:**
```bash
# Check all services status
docker compose ps

# Check logs for each service
docker compose logs frontend --tail 20
docker compose logs backend --tail 20
docker compose logs nginx --tail 10

# Look for specific error patterns:
# - 404s for JS files (/js/extensions.js, /components/...)
# - Webpack HMR connection errors
# - Upstream header errors in nginx
```

### 2. API Routing Problems
**Symptoms:**
- Frontend loads but can't communicate with backend
- API calls returning 404
- CORS errors

**Diagnostic Steps:**
```bash
# Test backend directly
curl -I http://localhost:BACKEND_PORT/api/health
curl http://localhost:BACKEND_PORT/api/

# Test frontend proxy routing
curl -I http://localhost:FRONTEND_PORT/api/endpoint

# Check Next.js rewrites configuration
cat frontend/next.config.ts
```

### 3. Build Cache Issues
**Symptoms:**
- Containers won't start after code changes
- Old code running despite rebuilds
- Build timeouts or failures

**Solution Pattern:**
```bash
# Clean slate approach
docker compose down
docker system prune -f  # Removes unused containers, networks, images
docker compose up --build -d

# For stubborn cases, nuclear option:
docker compose down --volumes
docker system prune -a  # WARNING: removes ALL unused images
```

## Systematic Troubleshooting Workflow

### Step 1: Service Health Check
```bash
# Start with basic service status
cd PROJECT_ROOT
docker compose ps

# Check each service individually
docker compose logs SERVICE_NAME --tail 20
```

### Step 2: Network Connectivity Test
```bash
# Test each service port directly
curl -I http://localhost:PORT

# For frontend apps, check if webpack dev server is responding
curl -I http://localhost:3000/_next/webpack-hmr
```

### Step 3: Environment Variable Validation
```bash
# Check if environment variables are properly set
docker compose config

# Look for missing or incorrect values, especially:
# - Database connection strings
# - API URLs
# - Service hostnames
```

### Step 4: Progressive Startup
When full stack fails, start services incrementally:

```bash
# 1. Start database first
docker compose up -d postgres
docker compose logs postgres

# 2. Start backend
docker compose up -d backend
docker compose logs backend --tail 10

# 3. Start frontend (often the problematic layer)
# Sometimes better to run outside Docker initially
cd frontend && npm install && npm run dev
```

### Step 5: Native vs Docker Comparison
If Docker setup fails, run components natively to isolate issues:

```bash
# Run frontend natively with proper backend URL
cd frontend
BACKEND_URL=http://localhost:BACKEND_PORT npm run dev

# This helps identify:
# - Whether issue is Docker networking or application code
# - Correct environment variable values
# - Working port configurations
```

## Next.js + Docker Specific Issues

### Missing JavaScript Files (404s)
**Root Cause:** Build artifacts not properly generated or mounted

**Solutions:**
1. **Check volume mounts:**
   ```yaml
   frontend:
     volumes:
       - ./frontend/src:/app/src
       - ./frontend/.next:/app/.next  # Critical for build artifacts
   ```

2. **Rebuild without cache:**
   ```bash
   docker compose build frontend --no-cache
   ```

3. **Run native build first:**
   ```bash
   cd frontend && npm run build
   ```

### Webpack HMR Connection Issues
**Symptoms:** `upstream sent no valid HTTP/1.0 header` in nginx logs

**Solutions:**
1. **Configure nginx for websockets:**
   ```nginx
   location /_next/webpack-hmr {
       proxy_pass http://frontend:3000;
       proxy_http_version 1.1;
       proxy_set_header Upgrade $http_upgrade;
       proxy_set_header Connection "upgrade";
   }
   ```

2. **Disable HMR in production-like setups:**
   ```dockerfile
   ENV NODE_ENV=development
   ENV FAST_REFRESH=false
   ```

### API Proxy Configuration
**Common Issue:** Frontend can't reach backend APIs

**Next.js rewrites pattern:**
```typescript
// next.config.ts
const nextConfig = {
  async rewrites() {
    return [
      { source: '/api/:path*', destination: `${BACKEND_URL}/api/:path*` }
    ];
  }
};
```

**Debugging steps:**
```bash
# 1. Verify backend is accessible
curl http://localhost:BACKEND_PORT/api/health

# 2. Test rewrite is working
curl http://localhost:FRONTEND_PORT/api/health

# 3. Check environment variable in container
docker exec FRONTEND_CONTAINER env | grep BACKEND_URL
```

## Port Conflict Resolution

### Finding and Killing Conflicting Processes
```bash
# Find what's using a port
lsof -i :3000

# Kill specific process
kill PID_NUMBER

# Kill all processes matching pattern
pkill -f "next dev"
```

### Clean Port Setup
```bash
# Ensure clean state
docker compose down
pkill -f "next dev"  # Kill any orphaned dev servers
lsof -ti:3000,5000,8080 | xargs kill  # Kill common dev ports

# Restart with clean slate
docker compose up -d
```

## Environment-Specific Configurations

### Local Development
```bash
# Often better to run frontend natively, backend in Docker
docker compose up -d postgres backend
cd frontend && BACKEND_URL=http://localhost:BACKEND_PORT npm run dev
```

### Network Access (iPad, mobile testing)
```bash
# Get local IP
ifconfig en0 | grep "inet " | awk '{print $2}'

# Ensure services bind to 0.0.0.0, not 127.0.0.1
docker compose ps  # Check port mappings

# Common Docker Compose port syntax
ports:
  - "8080:80"  # Binds to all interfaces
  - "127.0.0.1:8080:80"  # Only localhost (problematic for network access)
```

## Quick Diagnostic Commands

```bash
# Full stack health check
docker compose ps && curl -I http://localhost:3000 && curl -I http://localhost:5000/api/health

# Log monitoring
docker compose logs -f frontend | grep -E "(404|error|Error)"

# Network inspection
docker network ls
docker network inspect PROJECT_default
```

## When to Use This Skill

- Docker Compose services won't start or communicate
- Frontend loads but JavaScript/API calls fail
- "Website loads but nothing works" symptoms
- Need to enable network access from other devices
- Webpack/Next.js hot reload issues in Docker
- API proxy routing problems

**Success Indicators:**
- All services showing "healthy" status
- Frontend serves pages and assets (no 404s)
- API endpoints respond correctly
- Network access works from target devices