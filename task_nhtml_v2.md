# Roadmap Nhtml 2.0 — Transition Native

## 🎯 Phase 1-3 : Prototype Python & Headless (TERMINÉ)
- [x] Découpage AST JSON (Manifeste).
- [x] Micro-Runtime JS (Hydratation).
- [x] Support des expressions complexes (ternaires, filtres).
- [x] Extraction CSS automatique.
- [x] Validation sur `kitchen_sink.nhtml`.

## 🏗️ Phase 4 : Portage Natif (EN COURS)
- [ ] Choix de Rust comme langage de cœur.
- [ ] Développement du `nhtml-parser-rs`.
- [ ] Développement du `nhtml-runtime-rs` (Wasm).
- [ ] Pont DOM via `web_sys` / `js-sys`.
- [ ] Benchmarks comparatifs JS vs Wasm.

## 🚀 Phase 5 : Migration NCMS & Écosystème
- [ ] Re-compilation totale de NCMS en V2.
- [ ] Intégration du module `.so` dans le backend PHP (via extension FFI).
- [ ] Publication de la spec V2-VX stable.
