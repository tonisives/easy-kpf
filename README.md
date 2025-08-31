# Easy KPF - Kubernetes Port Forward Manager

A simple, intuitive application for managing Kubernetes port forwarding.

## Key Features

### One-Click Port Forwarding

- Start and stop port forwarding with a single click
- Visual status indicators show which services are running
- Support for multiple port mappings per service

### Configuration Management

- Save and manage port forwarding configurations
- Quick access to frequently used services
- Edit and update configurations on the fly

### Smart Kubernetes Integration

- Browse all available kubectl contexts
- Discover namespaces and services automatically
- Auto-detect service ports for easy setup

### Native Desktop Experience

- Fast, responsive native application built with Rust
- Clean, modern interface
- Cross-platform support (macOS, Windows, Linux)

### Developer Friendly

- No complex configuration files
- Works with existing kubectl setup
- Lightweight and resource-efficient

### Requirements

- kubectl installed and configured
- At least one Kubernetes context configured

### Quick Start

1. Launch the app
2. Select your Kubernetes context, namespace, and service
3. Configure port mappings
4. Click start to begin port forwarding
5. Access your services at `localhost:<port>`

## Build

```bash
pnpm tauri build
```

Perfect for developers working with Kubernetes who want a simple, reliable way to manage port forwarding without command-line complexity.
