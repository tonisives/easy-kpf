# Easy KPF - Kubernetes Port Forward Manager

A desktop application for managing Kubernetes port forwarding with a clean interface.

![Main Interface](etc/images/appstore_screenshots/one_1280x800.png)

## Features

- Visual port forwarding management
- Support for multiple Kubernetes contexts
- Drag and drop interface for organizing forwards
- Real-time status monitoring
- Persistent configuration storage

![Configuration](etc/images/appstore_screenshots/two_1280x800.png)

![Port Forwards](etc/images/appstore_screenshots/three_1280x800.png)

## Requirements

- kubectl installed and configured
- At least one Kubernetes context configured

## Installation

Built with Tauri (Rust + React). Cross-platform support for macOS, Windows, and Linux.

## Development

```bash
pnpm install
pnpm tauri dev
```

## Build

```bash
pnpm tauri build
```

## Configuration

Port forward configurations are stored in YAML files in the system config directory:

- **macOS**: `~/Library/Application Support/EasyKpf/`
- **Linux**: `~/.config/EasyKpf/`
- **Windows**: `%APPDATA%/EasyKpf/`

### Configuration Files

- `port-forwards.yaml` - Port forward definitions
- `app-config.yaml` - Application settings (kubectl path, kubeconfig path)

### Port Forward Configuration Structure

```yaml
configs:
  - name: "My Service"
    context: "minikube"
    namespace: "default"
    service: "my-service"
    ports: ["8080:80", "9090:9090"]
```

Configuration files are automatically created with defaults on first run.

## Technology

- Frontend: React + TypeScript + Vite
- Backend: Rust + Tauri
- Kubernetes integration via kubectl
