---
# 🔍 Audit — `.github/workflows/release.yml`
**Fichier d'audit** : `audit_03_release_yml.md`
**Date** : 29 Avril 2026
**Auditeur** : Claude Sonnet 4.6
**Fichier suivant** : `audit_04_bridge_js.md`

---

## 📋 Informations générales
| Champ | Valeur |
|-------|--------|
| Chemin | `.github/workflows/release.yml` |
| Rôle | CI/CD - Compilation et publication des releases |
| Lignes | 91 |
| Langage | YAML |
| Score Sécurité | 🟢 9/10 |
| Score Performance | 🟡 7/10 |

---

## 🟡 VULNÉRABILITÉS MOYENNES
> Corriger sous 1 mois

### MED-03-001 — Risque de "Supply Chain Attack" (SLSA)
- **Ligne(s)** : L.77-90
- **Type** : Supply Chain Integrity
- **Impact** : Bien que la permission `attestations: write` soit présente (L.9), le pipeline n'utilise pas d'action spécifique pour générer des preuves de provenance (SLSA) pour les binaires compilés. Si le runner GitHub est compromis, des binaires vérolés pourraient être distribués sans que les utilisateurs puissent vérifier cryptographiquement l'origine de la compilation.

**Solution recommandée** :
Ajouter une étape de génération de provenance SLSA via l'action officielle `actions/attest-build-provenance` après la création des artefacts.

---

## 🔵 PROBLÈMES DE PERFORMANCE
### PERF-03-001 — Absence de cache Cargo
- **Ligne(s)** : L.38-44
- **Type** : Temps de compilation (CI)
- **Impact** : L'action compile `sqlx`, `reqwest` et `zstd` (qui incluent du C et des macros lourdes) à chaque build depuis zéro. Cela ralentit significativement le temps d'exécution de la pipeline (plusieurs minutes perdues).

**Avant** :
```yaml
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      - name: Build Release
        run: cargo build --release
```

**Après** :
```yaml
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      - name: Cache Cargo
        uses: Swatinem/rust-cache@v2
      - name: Build Release
        run: cargo build --release
```

---

## ⚪ QUALITÉ & MAINTENABILITÉ
- [x] Fix récent : Le renommage de `nhtml-gateway` en `nhtml` a été corrigé, évitant l'échec silencieux/bloquant des builds.
- [ ] Dépendance à `sed` : L'extraction du Changelog (L.71-75) via `sed` est fragile si le format du `CHANGELOG.md` dévie légèrement.

---

## ✅ POINTS POSITIFS
- Permissions du GITHUB_TOKEN restreintes (`contents: write` et `attestations: write`) au lieu du permissif complet.
- Environnement matrix propre isolant macOS, Linux et Windows.

---

## 📊 Résumé du fichier
| Criticité | Nombre |
|-----------|--------|
| 🔴 Critiques | 0 |
| 🟠 Hautes | 0 |
| 🟡 Moyennes | 1 |
| 🔵 Performance | 1 |
| ⚪ Qualité | 1 |

---
*Fichier suivant → `audit_04_bridge_js.md`*
