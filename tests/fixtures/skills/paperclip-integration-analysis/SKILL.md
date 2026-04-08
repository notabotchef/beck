---
name: paperclip-integration-analysis
description: Analyze Paperclip multi-agent organizations and develop integration strategies with existing projects
version: 1.0.0
author: Hermes Agent
license: MIT
platforms: [linux, macos]
prerequisites:
  commands: [curl, pnpm]
  ports: [3100]
metadata:
  hermes:
    tags: [paperclip, multi-agent, orchestration, integration, analysis]
    homepage: https://github.com/paperclipai/paperclip
---

# Paperclip Integration Analysis

Use this skill when analyzing Paperclip multi-agent organizations and developing integration strategies with existing codebases.

## When to Use This Skill

- User has Paperclip running (localhost:3100) and wants to integrate it with another project
- Need to understand existing Paperclip org structure and identify integration opportunities
- Developing bridge architectures between Paperclip and external systems
- Planning specialized agent roles for domain-specific operations

## Discovery Methodology

### 1. Paperclip Organization Audit

Start by mapping the existing organization structure:

```bash
# Verify Paperclip is running
curl -s http://localhost:3100/api/health

# Get company info
curl -s http://localhost:3100/api/companies

# Map agent hierarchy 
curl -s "http://localhost:3100/api/companies/[COMPANY_ID]/agents"

# Review current work
curl -s "http://localhost:3100/api/companies/[COMPANY_ID]/issues"
```

Key information to extract:
- Agent roles and reporting structure
- Adapter types (claude_local, codex_local, opencode_local, etc.)
- Current issue status and priorities
- Budget and permission configurations

### 2. Codebase Context Analysis

Read the target project's documentation to understand:
- Architecture and tech stack
- Integration points and APIs
- Current pain points or blockers
- Domain-specific workflows

Always start with:
- README.md
- AGENTS.md (if present)
- Architecture documentation
- Recent session history for context

### 3. Integration Opportunity Identification

Look for these integration patterns:

**Bridge Adapters:**
- Create new adapter types that can interface with external systems
- Example: `carabiner-local` adapter for restaurant management integration

**Domain-Specific Agents:**
- Specialized agents for operational domains
- Example: Kitchen Manager, FOH Manager, Financial Controller agents

**Data Pipeline Integration:**
- Real-time data flow between Paperclip and external systems
- Action card generation from external events
- Autonomous response to system triggers

**Workflow Orchestration:**
- Multi-agent collaboration on complex tasks
- Parallel execution of related workstreams
- Cross-system task delegation

## Strategic Planning Framework

### Phase 1: Foundation
1. **Bridge Architecture**: Create adapters for system integration
2. **Basic Data Flow**: Establish read/write connectivity
3. **Proof of Concept**: Simple end-to-end workflow

### Phase 2: Specialization  
1. **Domain Agents**: Hire specialized agents for operational areas
2. **Real-time Integration**: Live data monitoring and response
3. **Autonomous Operations**: Routine task automation

### Phase 3: Optimization
1. **Performance Tuning**: Optimize agent coordination
2. **Advanced Workflows**: Complex multi-agent orchestration
3. **Scaling**: Expand to multiple domains/clients

## Technical Implementation Patterns

### Custom Adapter Development

```typescript
// packages/adapters/[system]-local/src/index.ts
export const type = "system_local";
export const label = "System Integration (local)";

export const models = [
  { id: "default", label: "Default Integration" }
];

export const agentConfigurationDoc = `
Core fields:
- systemUrl (string): target system API URL
- credentials (object): authentication config
- domain (string): operational domain scope
- permissions (array): allowed operations
`;
```

### Agent Role Specialization

Common patterns for specialized agents:
- **Operations Agents**: Domain experts (kitchen, finance, marketing)
- **Integration Agents**: System bridge specialists  
- **Coordination Agents**: Cross-domain workflow managers
- **Monitoring Agents**: Real-time alerting and response

### Data Flow Architecture

```
External System → Bridge Adapter → Paperclip Agent → Action/Response → External System
```

Key considerations:
- Real-time vs batch processing
- Error handling and retry logic
- Authentication and permissions
- Data transformation requirements

## Common Integration Challenges

### Authentication & Permissions
- Multi-system credential management
- Permission boundary enforcement
- API rate limiting coordination

### Data Synchronization
- Real-time vs eventual consistency
- Conflict resolution strategies
- Schema evolution handling

### Agent Coordination
- Task dependency management
- Resource contention resolution
- Failure propagation handling

## Success Metrics

**Technical Metrics:**
- Successful bridge adapter deployment
- End-to-end workflow completion
- System uptime and reliability

**Operational Metrics:**
- Task automation percentage
- Response time improvements
- Error rate reduction

**Business Metrics:**
- Process efficiency gains
- Cost reduction through automation
- Scalability improvements

## Notes

- Always start with the smallest viable integration
- Focus on high-impact, low-complexity workflows first
- Maintain clear separation between Paperclip orchestration and domain logic
- Document integration patterns for reuse across similar projects

## Example: Restaurant Management Integration

**Discovered Opportunity:** CarabinerOS + Paperclip
- **Bridge**: Agent Zero CLI integration via custom adapter
- **Specialization**: Kitchen/FOH/Financial agents for operations
- **Data Flow**: Restaurant database → AI analysis → Action cards
- **Automation**: 80% of routine tasks handled autonomously

**Key Insight:** Transform single-agent systems into multi-agent organizations for exponential scaling and domain expertise.