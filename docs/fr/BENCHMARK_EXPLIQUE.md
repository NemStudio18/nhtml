# 📊 Guide du Benchmark NHTML

Ce document explique les métriques fournies par la commande `nhtml bench` et la méthodologie utilisée pour l'évaluation des performances.

## 📈 Métriques Clés

### 1. Gain de Bande Passante
Comparaison entre la taille du HTML brut et le package binaire NHTML optimisé (après compression B-TREE et Zstd optionnel).
*   **Formule** : `(1 - (Taille_NHTML / Taille_HTML)) * 100`
*   **Importance** : Un gain élevé signifie des chargements plus rapides et moins de consommation de données, particulièrement sur mobile.

### 2. Facteur d'Efficacité
Représente combien de fois NHTML est plus efficace que le HTML brut.
*   **Formule** : `Taille_HTML / Taille_NHTML`
*   **Objectif** : Les projets industriels visent généralement une efficacité > 2.0x.

### 3. Latence Réseau Sauvée (Estimée)
Temps théorique économisé lors de la transmission réseau sur une connexion limitée (ex: 1Mbps).
*   **Formule** : `(Taille_HTML - Taille_NHTML) / (Vitesse_Lien / 8)`
*   **Signification** : Quelques millisecondes sauvées améliorent le SEO et la rétention utilisateur (First Contentful Paint).

### 4. Score de Complexité CPU
Score arbitraire estimant la complexité de la sérialisation du DOM vers le format binaire B-TREE.
*   **Formule** : `Taille_NHTML_Brut / 1000` (CPU-ops/pkt)
*   **Cible** : Plus le score est bas, mieux c'est. NHTML est conçu pour rester sous 1.0 pour garantir des mises à jour sans latence sur la Gateway.

## 🛠️ Méthodologie

L'outil de benchmark simule une compilation de production complète :
1.  **Parsing** : Le HTML est analysé en un arbre `NodeSpec`.
2.  **Extraction d'état** : Les nœuds réactifs (avec attributs `n-`) sont identifiés.
3.  **Sérialisation Binaire** : L'arbre est converti en paquets binaires NBPS v0.6.0.
4.  **Compression** : Le payload est compressé via Zstd pour simuler le poids réel en production.

## 🚀 Interpréter les Résultats

| Efficacité | Qualité | Action |
| :--- | :--- | :--- |
| **< 1.0x** | Médiocre | Vérifiez si la page contient trop d'éléments non-réactifs. |
| **1.0x - 1.5x** | Bon | Standard pour les petites pages. |
| **> 2.0x** | Excellent | Utilisation optimale du B-TREE et de la compression. |
| **> 5.0x** | Ultra | Applications réactives haute densité (Tableaux de bord, etc). |
