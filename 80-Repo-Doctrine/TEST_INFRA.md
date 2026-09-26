---
title: "E2E Test Infra: Kovanica Map Logic Subsystem"
category: 80-Repo-Doctrine
source: TEST_INFRA.md
synced: 2026-09-26
---
# E2E Test Infra: Kovanica Map Logic Subsystem

## Test Philosophy
- Opaque-box, requirement-driven testing. Validates behavior from user/component contract perspective.
- Methodology: Category-Partition + Boundary Value Analysis (BVA) + Pairwise Combinations + Real-World Workloads.
- Minimum Thresholds for 17 Features:
  - Tier 1 (Feature Coverage): ≥5 tests per feature (≥85 tests)
  - Tier 2 (Boundary & Corner Cases): ≥5 tests per feature (≥85 tests)
  - Tier 3 (Cross-Feature Combinations): ≥17 pairwise tests
  - Tier 4 (Real-World Scenarios): ≥9 realistic user workflow scenarios
  - Total Target: ≥196 tests across the test suite.

## Feature Inventory
| # | Feature | Source | Tier 1 | Tier 2 | Tier 3 | Tier 4 |
|---|---------|--------|:------:|:------:|:------:|:------:|
| 1 | Global Geo Catalog & Enriched Metadata | PROJECT.md §1 | 5 | 5 | ✓ | ✓ |
| 2 | ISO Code Conversion & Helpers | PROJECT.md §2 | 5 | 5 | ✓ | ✓ |
| 3 | Multi-Scale Engine & Palettes | PROJECT.md §3 | 5 | 5 | ✓ | ✓ |
| 4 | Origin Inference Synchronization | PROJECT.md §4 | 5 | 5 | ✓ | ✓ |
| 5 | Multi-Projection Engine | PROJECT.md §5 | 5 | 5 | ✓ | ✓ |
| 6 | TopoJSON Interior Mesh Borders | PROJECT.md §6 | 5 | 5 | ✓ | ✓ |
| 7 | Disjoint Centroids & Pulse Beacon | PROJECT.md §7 | 5 | 5 | ✓ | ✓ |
| 8 | Smooth D3 Animated Camera | PROJECT.md §8 | 5 | 5 | ✓ | ✓ |
| 9 | Keyboard Navigation & a11y | PROJECT.md §9 | 5 | 5 | ✓ | ✓ |
| 10 | Dynamic Metric Switcher | PROJECT.md §10 | 5 | 5 | ✓ | ✓ |
| 11 | Temporal / Time-Range Filters | PROJECT.md §11 | 5 | 5 | ✓ | ✓ |
| 12 | Multi-Metric Data Reconciliation | PROJECT.md §12 | 5 | 5 | ✓ | ✓ |
| 13 | Interactive Discrete Legend | PROJECT.md §13 | 5 | 5 | ✓ | ✓ |
| 14 | Search Dimming & Map Highlighting | PROJECT.md §14 | 5 | 5 | ✓ | ✓ |
| 15 | Continent Quick-Jump Presets | PROJECT.md §15 | 5 | 5 | ✓ | ✓ |
| 16 | Mobile Sheet / Drawer Inspection | PROJECT.md §16 | 5 | 5 | ✓ | ✓ |
| 17 | E2E Testing Suite & CI Validation | PROJECT.md §17 | 5 | 5 | ✓ | ✓ |

## Test Architecture
- **Runner**: Node.js native `node:test` + `node:assert/strict` with `jiti` for instant TypeScript & TSX compilation.
- **Test Files**:
  - `kovanica-web-src/test/geo-catalog.test.mjs`
  - `kovanica-web-src/test/geo-scale.test.mjs`
  - `kovanica-web-src/test/geo-infer.test.mjs`
  - `kovanica-web-src/test/world-choropleth.test.mjs`
  - `kovanica-web-src/test/e2e-map-workflow.test.mjs`
- **Execution Command**:
  ```bash
  export PATH="$HOME/.nvm/versions/node/v20.20.2/bin:$PATH" && cd /root/kovanica-web-src && node --test test/*.test.mjs
  ```

## Real-World Application Scenarios (Tier 4)
| # | Scenario | Features Exercised | Complexity |
|---|----------|--------------------|------------|
| 1 | First-time visitor lands from Europe, pulses origin, views provenance DAG | F1, F4, F7, F8, F12 | High |
| 2 | Network operator switches between Mined Blocks, Nodes, and Pulses over 24h/7d | F3, F10, F11, F12, F13 | High |
| 3 | Analyst switches between NaturalEarth1, EqualEarth, Mercator, and Orthographic globe projections | F5, F6, F7, F8 | High |
| 4 | Power user searches country in ranking, verifies map highlighting and focusing | F1, F2, F8, F14 | Medium |
| 5 | Mobile user opens map, selects country via touch, opens bottom sheet inspector | F1, F2, F7, F16 | High |
| 6 | Keyboard-only accessibility user tabs across countries and navigates via arrow keys | F8, F9 | High |
| 7 | Analyst inspects logarithmic vs quantile distribution for extreme power-law network activity | F3, F10, F13 | High |
| 8 | Multi-territory country inspection (USA, France, Norway) centroid & bounding box verification | F1, F5, F7, F8 | Medium |
| 9 | High-density data offline-to-online reconciliation under network latency | F1, F4, F12 | High |
