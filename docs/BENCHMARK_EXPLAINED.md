# 📊 NHTML Benchmark Guide

This document explains the metrics provided by the `nhtml bench` command and the methodology used for performance evaluation.

## 📈 Core Metrics

### 1. Bandwidth Gain (Gain de bande passante)
Calculated by comparing the raw HTML size with the optimized NHTML binary package (after B-TREE compression and optional Zstd).
*   **Formula**: `(1 - (NHTML_Size / HTML_Size)) * 100`
*   **Why it matters**: Higher gains mean faster page loads and less data usage, especially on mobile networks.

### 2. Efficiency Factor (Facteur d'efficacité)
Represents how many times more efficient NHTML is compared to raw HTML.
*   **Formula**: `HTML_Size / NHTML_Size`
*   **Goal**: Industrial projects usually aim for > 2.0x efficiency.

### 3. Estimated Latency Savings (Latence réseau sauvée)
Theoretical time saved during network transmission on a limited connection (e.g., 1Mbps).
*   **Formula**: `(HTML_Size - NHTML_Size) / (Link_Speed / 8)`
*   **Significance**: Even a few milliseconds saved can improve SEO and user retention (First Contentful Paint).

### 4. CPU Complexity Score (Charge CPU Sérialesation)
An arbitrary score estimating the complexity of serializing the DOM into the binary B-TREE format.
*   **Formula**: `NHTML_Raw_Size / 1000` (CPU-ops/pkt)
*   **Target**: Lower is better. NHTML is designed to keep this below 1.0 for standard pages to ensure zero-latency updates on the Gateway.

## 🛠️ Methodology

The benchmark tool simulates a complete production compilation:
1.  **Parsing**: The HTML is parsed into a `NodeSpec` tree.
2.  **State Extraction**: Reactive nodes (with `n-` attributes) are identified.
3.  **Binary Serialization**: The tree is converted into NBPS v0.6.0 binary packets.
4.  **Compression**: The payload is compressed using Zstd (if available) to simulate final production payload.

## 🚀 How to interpret results

| Efficiency | Quality | Action |
| :--- | :--- | :--- |
| **< 1.0x** | Poor | Check if the page has too many non-reactive elements. |
| **1.0x - 1.5x** | Good | Standard for small pages. |
| **> 2.0x** | Excellent | Optimal use of B-TREE and compression. |
| **> 5.0x** | Ultra | High-density reactive applications (Dashboards, etc). |
