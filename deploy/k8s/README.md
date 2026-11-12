# Kovanica Kubernetes & Helm Deployment Guide

Production-ready Kubernetes manifests and Helm chart for operating Kovanica (KUNA) blockchain nodes.

---

## Architecture Overview

Kovanica nodes operate as containerized services optimized for standard VPS/cloud instances (2–4 GB RAM, 2 CPU cores). In Kubernetes, nodes are deployed as a `StatefulSet` to ensure stable network identities and persistent chain data storage across pod restarts.

### Ports & Protocols

| Port | Protocol | Purpose | Access Level |
|---|---|---|---|
| **8000** | TCP | Libp2p P2P Gossipsub & Block Sync | Public / NodePort / LoadBalancer |
| **8545** | TCP | JSON-RPC API | Internal ClusterIP / Optional Ingress |
| **8546** | TCP | WebSocket Event Subscriptions | Internal ClusterIP / Optional Ingress |
| **9100** | TCP | Prometheus Metrics Endpoint | ClusterIP / ServiceMonitor |

---

## Directory Structure

```text
deploy/
├── k8s/                           # Standard Kubernetes manifests
│   ├── configmap.yaml             # Node configuration (chain ID, logging, P2P)
│   ├── service-p2p.yaml           # Headless discovery + external P2P NodePort
│   ├── service-rpc.yaml           # ClusterIP service for JSON-RPC, WS & Metrics
│   ├── servicemonitor.yaml        # Prometheus Operator ServiceMonitor
│   ├── statefulset.yaml           # StatefulSet deploying kovanica-node with PVC
│   └── README.md                  # This deployment manual
└── helm/
    └── kovanica/                  # Production Helm chart (v0.2.0)
        ├── Chart.yaml             # Chart metadata
        ├── values.yaml            # Default deployment configuration
        └── templates/             # Parameterized Kubernetes templates
```

---

## Method 1: Deployment via `kubectl` Manifests

### Prerequisites
- Kubernetes cluster (v1.26+)
- `kubectl` configured with cluster admin or namespace access
- A default `StorageClass` supporting dynamic volume provisioning (e.g., standard, gp3, local-path)

### Step 1: Deploy Configuration and Services

```bash
kubectl apply -f deploy/k8s/configmap.yaml
kubectl apply -f deploy/k8s/service-p2p.yaml
kubectl apply -f deploy/k8s/service-rpc.yaml
```

### Step 2: (Optional) Apply Prometheus ServiceMonitor

If the cluster runs Prometheus Operator:

```bash
kubectl apply -f deploy/k8s/servicemonitor.yaml
```

### Step 3: Deploy the Kovanica Node StatefulSet

```bash
kubectl apply -f deploy/k8s/statefulset.yaml
```

### Step 4: Verify Node Status

```bash
# Check StatefulSet and Pod status
kubectl get statefulset kovanica-node
kubectl get pods -l app.kubernetes.io/name=kovanica

# View startup logs
kubectl logs -f kovanica-node-0
```

---

## Method 2: Deployment via Helm Chart

The Helm chart in `deploy/helm/kovanica/` provides automated parameterization for different networks (Mainnet, Testnet, Hybrid Testnet) and node roles (block author, RPC, Seed). `validator` in the role value names a PoW block-authoring node on chains 1 and 2; there is no validator committee.

### 1. Basic Installation (Default: Mainnet Block Author)

```bash
helm install kovanica deploy/helm/kovanica
```

### 2. Configuration for Specific Environments

#### Canonical Testnet (Chain ID 2)
```bash
helm install kovanica-testnet deploy/helm/kovanica \
  --set node.chainId=2 \
  --set image.tag="v0.2.0"
```

#### Hybrid PoW Testnet (Chain ID 2 with Mining Enabled)
```bash
helm install kovanica-hybrid deploy/helm/kovanica \
  --set node.chainId=2 \
  --set mining.enabled=true \
  --set mining.threads=2
```

#### Public RPC Node with Ingress
```bash
helm install kovanica-rpc deploy/helm/kovanica \
  --set node.role=rpc \
  --set ingress.enabled=true \
  --set ingress.className="nginx" \
  --set "ingress.hosts[0].host=rpc.kovanica.example.com" \
  --set "ingress.hosts[0].paths[0].path=/" \
  --set "ingress.hosts[0].paths[0].pathType=Prefix"
```

### 3. Key `values.yaml` Parameters

| Parameter | Default | Description |
|---|---|---|
| `image.repository` | `kovanica/kovanica-node` | Docker image repository |
| `image.tag` | `v0.2.0` | Node software version tag |
| `node.chainId` | `1` | Chain ID: `1` (Mainnet), `2` (Testnet), `5` (Hybrid Testnet) |
| `node.role` | `validator` | Node role: `validator` (= PoW block author on chains 1/2), `seed`, or `rpc` |
| `node.targetBlockMs` | `4000` | Target block time in milliseconds (4 s) |
| `node.maxPeers` | `50` | Maximum connected libp2p peers |
| `mining.enabled` | `false` | Enable KovanicaHash PoW miner |
| `resources.requests.cpu` | `1000m` | CPU request (1 core) |
| `resources.requests.memory`| `2Gi` | Memory request (2 GB RAM) |
| `resources.limits.cpu` | `2000m` | CPU limit (2 cores) |
| `resources.limits.memory` | `4Gi` | Memory limit (4 GB RAM) |
| `persistence.size` | `50Gi` | PVC storage size for chain data |
| `serviceMonitor.enabled` | `true` | Create Prometheus Operator ServiceMonitor |

---

## Verifying Node Sync and Connectivity

### 1. Port-Forward JSON-RPC Locally

```bash
kubectl port-forward svc/kovanica-rpc 8545:8545 &
```

### 2. Inspect Sync State via CLI

```bash
# Query node sync state
kovanica node sync --rpc-url http://127.0.0.1:8545

# Query connected peers
kovanica node peers --rpc-url http://127.0.0.1:8545
```

### 3. Check Prometheus Metrics

```bash
kubectl port-forward svc/kovanica-rpc 9100:9100 &
curl -s http://127.0.0.1:9100/metrics | grep kovanica_
```
