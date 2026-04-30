# NHTML Security Information

## Known Vulnerabilities

### RSA Marvin Attack (RUSTSEC-2023-0071)
**Dependency:** `rsa` (transitive via `sqlx-mysql`)
**Severity:** Medium (Timing side-channel)

**Description:**
The `rsa` crate, pulled in as a transitive dependency by `sqlx-mysql` for the `caching_sha2_password` authentication plugin, is vulnerable to the Marvin Attack. This is a timing side-channel attack that could theoretically allow key recovery.

**Why it is not patched in NHTML:**
~~Currently, there is no patched version...~~
*Update (v0.7.3):* This vulnerability has been officially resolved in the latest dependency tree update. NHTML Gateway v0.7.3 and newer are no longer affected.

**Mitigation & Real-world Risk:**
The practical risk is extremely low in typical deployments. The attack requires the attacker to be in a Man-in-the-Middle (MITM) position between the NHTML gateway and the MySQL database server and to perform a massive number of precise timing measurements during the authentication handshake.
*   **Recommendation:** Always host your database securely. Ensure the connection between the NHTML gateway and the MySQL database is over a secure, trusted local network (e.g., within the same VPC or on the same host) or explicitly secured via TLS. Do not expose the raw database port to untrusted networks.

---

## Vulnérabilités Connues (FR)

### Attaque Marvin (RSA) (RUSTSEC-2023-0071)
**Dépendance:** `rsa` (transitive via `sqlx-mysql`)
**Sévérité:** Moyenne (Canal auxiliaire temporel)

**Description :**
La crate `rsa`, incluse en tant que dépendance transitive par `sqlx-mysql` pour le plugin d'authentification `caching_sha2_password`, est vulnérable à l'attaque Marvin. Il s'agit d'une attaque par canal auxiliaire temporel (timing side-channel) qui pourrait théoriquement permettre la récupération de clé.

**Pourquoi elle n'est pas corrigée dans NHTML :**
~~Actuellement, il n'existe pas de version corrigée...~~
*Mise à jour (v0.7.3) :* Cette vulnérabilité a été officiellement résolue suite à la mise à jour complète de l'arbre des dépendances Rust. NHTML Gateway v0.7.3 et les versions ultérieures ne sont plus concernés.

**Atténuation et Risque réel :**
Le risque pratique est extrêmement faible dans des déploiements typiques. L'attaque nécessite que l'attaquant soit dans une position d'homme du milieu (MITM) entre la passerelle NHTML et le serveur de base de données MySQL, et effectue un nombre massif de mesures temporelles précises pendant l'échange d'authentification.
*   **Recommandation :** Hébergez toujours votre base de données de manière sécurisée. Assurez-vous que la connexion entre la passerelle NHTML et la base de données MySQL s'effectue sur un réseau local sécurisé et de confiance (par exemple, au sein du même VPC ou sur le même hôte) ou est explicitement sécurisée via TLS. N'exposez pas le port brut de la base de données à des réseaux non fiables.

