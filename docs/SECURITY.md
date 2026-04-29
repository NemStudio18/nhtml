# NHTML Security Information

## Known Vulnerabilities

### RSA Marvin Attack (RUSTSEC-2023-0071)
**Dependency:** `rsa` (transitive via `sqlx-mysql`)
**Severity:** Medium (Timing side-channel)

**Description:**
The `rsa` crate, pulled in as a transitive dependency by `sqlx-mysql` for the `caching_sha2_password` authentication plugin, is vulnerable to the Marvin Attack. This is a timing side-channel attack that could theoretically allow key recovery.

**Why it is not patched in NHTML:**
Currently, there is no patched version of the `rsa` crate in the `0.9.x` series compatible with `sqlx` 0.8's dependencies. 
We have chosen to retain the `mysql` and `postgres` features in `sqlx` to ensure NHTML can be deployed in production environments using these databases for session management. Removing these features would severely limit the framework's capabilities.

**Mitigation & Real-world Risk:**
The practical risk is extremely low in typical deployments. The attack requires the attacker to be in a Man-in-the-Middle (MITM) position between the NHTML gateway and the MySQL database server and to perform a massive number of precise timing measurements during the authentication handshake.
*   **Recommendation:** Always host your database securely. Ensure the connection between the NHTML gateway and the MySQL database is over a secure, trusted local network (e.g., within the same VPC or on the same host) or explicitly secured via TLS. Do not expose the raw database port to untrusted networks.
