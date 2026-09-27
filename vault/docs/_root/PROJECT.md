# Project: Kovanica Map Logic & Choropleth Visualization Subsystem

## Architecture
- **Web Client**: TanStack Start / Vite / Nitro / React 19 / Tailwind CSS v4 in `kovanica-web-src/`.
- **Map Visualization Core**:
  - `src/components/map/world-choropleth.tsx`: D3 vector projection, SVG rendering, zoom/pan engine, interactive hover/selection.
  - `src/components/map/dashboard.tsx`: Top-level coordinator of map state, metrics, sidebar, and overlays.
  - `src/components/map/metric-switcher.tsx`: Metric and time-range selection controls.
  - `src/components/map/legend.tsx`: Discrete / continuous color scale legend with interactive bucket filtering.
  - `src/components/map/zoom-controls.tsx`: Floating map zoom, reset, and continent preset controls.
  - `src/components/map/country-panel.tsx`: Detail inspector, provenance DAG, regional breakdown, and global summary.
  - `src/components/map/ranking.tsx`: Filterable leaderboard with search synchronization to map polygons.
  - `src/components/map/origin-dag.tsx` & `region-bars.tsx`: Provenance breadcrumb hierarchy and continent aggregations.
- **Geospatial & Metric Data Layer**:
  - `src/lib/geo/catalog.ts`: Comprehensive ISO 3166-1 country catalog (~240 countries/territories), flag emojis, coordinates, ISO conversions, pulse/metric merging.
  - `src/lib/geo/scale.ts`: Multi-scale engine (Quantile, Quantize, Logarithmic, Linear, Threshold) with break calculations.
  - `src/lib/geo/metrics.ts`: Thematic palettes (Emerald Cyber, Kovanica Gold, Cyberpunk Neon, Viridis, Accessible High-Contrast), formatters.
  - `src/lib/geo/infer-origin.ts`: Privacy-preserving IANA timezone and browser locale origin resolver.
- **Testing & Tooling**:
  - Node 20 runtime + `node:test` + `jiti` loader for zero-dependency high-speed TypeScript/TSX component and unit testing.

## Feature Inventory
| # | Feature | Description | Milestone | Source |
|---|---------|-------------|-----------|--------|
| 1 | Global Geo Catalog & Enriched Metadata | Complete ~240 country catalog with ISO3, ISO2, Numeric, Flag Emojis, Coordinates, Regions | M1 | Survey E1/E2/E3 |
| 2 | ISO Code Conversion & Helpers | Bidirectional lookups, flag emoji generator, fallback resilience, country metadata lookup | M1 | Survey E1 |
| 3 | Multi-Scale Engine & Palettes | Quantile, Quantize, Logarithmic, Linear scales + Emerald, Gold, Viridis, Accessible palettes | M1 | Survey E1/E2 |
| 4 | Origin Inference Synchronization | Robust timezone & locale resolver unified with canonical catalog | M1 | Survey E1/E2 |
| 5 | Multi-Projection Engine | Support for NaturalEarth1, EqualEarth, Mercator, and Orthographic (Globe) projections | M2 | Survey E1 |
| 6 | TopoJSON Interior Mesh Borders | Clean non-overlapping shared borders via `topojson.mesh` + exterior coastlines | M2 | Survey E1 |
| 7 | Disjoint Centroids & Pulse Beacon | Area-weighted centroid calculation for accurate pulse beacon positioning on multi-polygons | M2 | Survey E1 |
| 8 | Smooth D3 Animated Camera | Animated transitions for zoom in/out, reset, and adaptive country bounding-box focusing | M2 | Survey E1 |
| 9 | Keyboard Navigation & a11y | `tabIndex={0}`, Enter/Space selection, Arrow key panning, ARIA live region stats | M2 | Survey E1 |
| 10 | Dynamic Metric Switcher | Interactive controls to switch between Pulses, Mined Blocks, P2P Nodes, and Normalized metrics | M3 | Survey E1/E2 |
| 11 | Temporal / Time-Range Filters | 24h, 7d, 30d, All-Time time-range aggregation support | M3 | Survey E1/E2 |
| 12 | Multi-Metric Data Reconciliation | Dual-source local & remote reconciliation for all supported metric types | M3 | Survey E2 |
| 13 | Interactive Discrete Legend | Discrete bucket swatches, range intervals, "No Data" swatch, and hover/click bucket filtering | M4 | Survey E1 |
| 14 | Search Dimming & Map Highlighting | Live synchronization between ranking search query and map polygon highlighting/dimming | M4 | Survey E1/E2 |
| 15 | Continent Quick-Jump Presets | Zoom control buttons for Europe, Americas, Asia-Pacific, Africa, Global reset | M4 | Survey E1 |
| 16 | Mobile Sheet / Drawer Inspection | Responsive bottom sheet / drawer (`vaul`) for country inspection on mobile viewports | M4 | Survey E1/E2 |
| 17 | E2E Testing Suite & CI Validation | 4-Tier comprehensive automated test suite (Tiers 1–4) + clean npm test script | M5 / Test Track | Survey E3 |

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | Core Geo Engine & Catalog | Global catalog (~240 nations), ISO conversions, flags, multi-scale engine, palettes, origin inference | none | PLANNED |
| M2 | Vector Map Engine & Projections | Multi-projection (NaturalEarth1, EqualEarth, Mercator, Orthographic), mesh borders, centroids, smooth zoom, a11y | M1 | PLANNED |
| M3 | Metric Switcher & Temporal Filtering | Dynamic metric switcher, time-range filtering, multi-metric feeds & reconciliation | M1, M2 | PLANNED |
| M4 | UI Overlays, Interactive Legend & Mobile | Discrete legend with bucket filtering, search highlight dimming, continent presets, mobile drawer | M2, M3 | PLANNED |
| M5 | E2E Test Suite Pass & Adversarial Hardening | Run 100% E2E test suite (Tiers 1-4), adversarial test pass (Tier 5), typecheck, lint, build:vps, audit | M1, M2, M3, M4 | PLANNED |

## Interface Contracts
### Geo Catalog (`src/lib/geo/catalog.ts`)
```typescript
export type RegionId = "Americas" | "Europe" | "Asia-Pacific" | "Middle East & Africa";

export type CountryMeta = {
  iso3: string;         // 3-letter alpha (e.g. "USA", "HRV")
  iso2: string;         // 2-letter alpha (e.g. "US", "HR")
  isoNumeric: string;   // 3-digit padded (e.g. "840", "191")
  name: string;         // Full English name
  region: RegionId;     // Continent grouping
  flagEmoji: string;    // Unicode flag emoji (e.g. "🇺🇸", "🇭🇷")
  capital?: string;     // Capital city
  latLng?: [number, number]; // [Latitude, Longitude]
  population?: number;  // Approx population for per-capita normalization
};

export type CountryView = CountryMeta & {
  pulses: number;
  value?: number;       // Value for currently selected metric
  formattedValue?: string;
};
```

### Scale & Metric Types (`src/lib/geo/scale.ts`, `src/lib/geo/metrics.ts`)
```typescript
export type ScaleType = "quantile" | "quantize" | "log" | "linear" | "threshold";
export type PaletteId = "emerald" | "gold" | "cyberpunk" | "viridis" | "accessible";

export type ColorScale = {
  color: (value: number | undefined) => string;
  breaks: number[];
  stops: readonly string[];
  emptyColor: string;
  type: ScaleType;
  palette: PaletteId;
  formattedRanges?: { min: number; max: number; label: string; color: string }[];
};

export type MetricType = "pulses" | "blocks" | "nodes" | "perCapita";
export type TimeRange = "24h" | "7d" | "30d" | "all";
```

### Map Component & Handle (`src/components/map/world-choropleth.tsx`)
```typescript
export type ProjectionType = "naturalEarth1" | "equalEarth" | "mercator" | "orthographic";

export type MapHandle = {
  zoomIn: () => void;
  zoomOut: () => void;
  reset: () => void;
  focusCountry: (isoNumeric: string) => void;
  setProjection: (proj: ProjectionType) => void;
  focusContinent: (region: RegionId) => void;
};

export type Props = {
  rows: CountryView[];
  selectedId: string | null;
  originId?: string | null;
  onSelect: (isoNumeric: string | null) => void;
  projection?: ProjectionType;
  scaleType?: ScaleType;
  palette?: PaletteId;
  highlightFilter?: (isoNumeric: string) => boolean;
  className?: string;
};
```

## Code Layout
- `kovanica-web-src/src/components/map/`
  - `world-choropleth.tsx`: Vector D3 map, SVG layers, projection, zoom, mesh borders, keyboard a11y.
  - `dashboard.tsx`: Map page state coordinator, metrics selection, reconciliation, drawer integration.
  - `metric-switcher.tsx`: Interactive metric & time-range controls.
  - `legend.tsx`: Discrete bucket legend with interactive hover/click filtering.
  - `zoom-controls.tsx`: Zoom buttons, reset, continent jump presets, projection switcher.
  - `country-panel.tsx`: Country inspection card, flag emoji, rank, share, provenance DAG.
  - `ranking.tsx`: Filterable leaderboard with flag emojis and search filter.
  - `origin-dag.tsx`: Breadcrumb provenance hierarchy.
  - `region-bars.tsx`: Continent aggregation distribution bars.
- `kovanica-web-src/src/lib/geo/`
  - `catalog.ts`: Complete ~240 country catalog, ISO conversions, flag emojis, merge functions.
  - `scale.ts`: Multi-scale calculations (Quantile, Quantize, Log, Linear, Threshold).
  - `metrics.ts`: Palette definitions, formatters, normalization functions.
  - `infer-origin.ts`: Timezone & locale origin resolver.
- `kovanica-web-src/test/`
  - `geo-catalog.test.mjs`: Catalog, ISO conversion, and pulse merging unit tests.
  - `geo-scale.test.mjs`: Multi-scale engine and palette unit tests.
  - `geo-infer.test.mjs`: Timezone and locale origin inference tests.
  - `world-choropleth.test.mjs`: Map topology, projections, mesh borders, and SSR tests.
  - `e2e-map-workflow.test.mjs`: 4-Tier E2E workflow and user scenario test suite.
