# Credential Storage — Platform-Independent, Encrypted

> Research notes for securely storing the Ghost Content API key and Admin API key in `ghost-io-api`.

---

## Requirements

| Requirement | Detail |
|-------------|--------|
| Works on all platforms | Linux, macOS, Windows, Redox — no OS daemon, no native keychain required |
| Encrypted at rest | Credentials are never stored in plaintext |
| Memory-safe | Keys are zeroed from memory when dropped |
| No external services | Pure Rust, file-based — usable in CI, containers, and headless servers |

---

## Why Not OS Keychains?

| Store | Platform | Problem |
|-------|----------|---------|
| macOS Keychain | macOS only | Not portable |
| Windows Credential Manager | Windows only | Not portable |
| libsecret / KWallet | Linux (daemon required) | Unavailable in containers, CI, headless servers |
| `keyring-core` | All of the above | Still delegates to OS stores; silently unavailable on some Linux environments |

**Conclusion:** OS keychains are not reliably available on all platforms. An encrypted file-based store is the only option that works everywhere with no external dependencies.

---

## Recommended Approach: Encrypted Credential File

### Crates Required

```toml
[dependencies]
# Encryption
chacha20poly1305 = "0.10"   # XChaCha20-Poly1305 AEAD — authenticated encryption, no padding, nonce-misuse resistant
argon2           = "0.5"    # Argon2id KDF — derives a 32-byte key from a passphrase + random salt

# In-memory secret protection
secrecy          = "0.10"   # Secret<T> wrapper — limits exposure, prevents accidental logging/copying
zeroize          = "1.8"    # Zeroize trait — securely wipes memory on drop (uses write_volatile + fence)

# File location
dirs             = "6.0"    # Cross-platform config directory: ~/.config (Linux), ~/Library/Application Support (macOS), %AppData% (Windows)

# Serialization
serde            = { version = "1", features = ["derive"] }
serde_json       = "1"

# Random nonce/salt generation
rand             = "0.8"
```

### Crate Roles

| Crate | Role | Notes |
|-------|------|-------|
| `chacha20poly1305` | AEAD encryption of the credentials blob | XChaCha20-Poly1305 variant — 192-bit nonce, no nonce collision risk, NCC Group audited |
| `argon2` | Passphrase → 32-byte encryption key | Argon2id (hybrid), winner of Password Hashing Competition; memory-hard, GPU/ASIC-resistant |
| `secrecy` | `Secret<String>` in-memory wrapper | Access only via `expose_secret()` — prevents accidental `Debug`/`Display` leakage |
| `zeroize` | Zero-on-drop for key material | Uses `core::ptr::write_volatile` + memory fences; survives compiler optimisations |
| `dirs` | Resolve platform config directory | XDG on Linux, `Application Support` on macOS, `AppData\Roaming` on Windows |

---

## File Format

The credential file (e.g. `~/.config/ghost-io-api/credentials`) stores:

```
[ 16 bytes — Argon2 salt (random, stored plaintext) ]
[ 24 bytes — XChaCha20 nonce (random, stored plaintext) ]
[ N bytes  — ciphertext (authenticated) ]
```

The ciphertext decrypts to a JSON object:

```json
{
  "content_api_key": "22444f78447824223cefc48062",
  "admin_api_key":   "64a33b0a7e23d8:abcdef1234567890abcdef1234567890"
}
```

The salt and nonce are never secret — they are randomly generated per write and are required for decryption.

---

## Key Derivation (Argon2id)

```
passphrase (user-supplied) + salt (16 bytes, random) → 32-byte encryption key
```

Argon2id parameters (OWASP recommended minimums):

| Parameter | Value | Notes |
|-----------|-------|-------|
| Algorithm | Argon2id | Hybrid — resists both side-channel and GPU attacks |
| Memory | 64 MiB (`m_cost = 65536`) | Makes brute force memory-expensive |
| Iterations | 3 (`t_cost = 3`) | Time cost |
| Parallelism | 1 (`p_cost = 1`) | Single-threaded derivation |
| Output length | 32 bytes | Matches XChaCha20-Poly1305 key size |

The passphrase can come from:
- An environment variable (`GHOST_CREDENTIALS_PASSPHRASE`) — for CI/automated use
- A CLI prompt at first run — for interactive/developer use

---

## Encryption (XChaCha20-Poly1305)

- **Algorithm:** XChaCha20-Poly1305 (RFC 8439 variant with extended 192-bit nonce)
- **Key:** 32 bytes, derived by Argon2id
- **Nonce:** 24 bytes, randomly generated per write (never reuse a nonce)
- **Authentication:** Poly1305 MAC covers the ciphertext — any tampering is detected on decryption
- **Pure Rust:** No FFI, no OS-specific code, compiles to any target including WASM

---

## In-Memory Key Protection

```rust
use secrecy::{Secret, ExposeSecret};
use zeroize::Zeroize;

// Keys held as Secret<String> — not Debug, not Clone
pub struct GhostCredentials {
    pub content_api_key: Secret<String>,
    pub admin_api_key:   Secret<String>,
}

// Only expose when actually needed (e.g. to sign a request)
let key = credentials.content_api_key.expose_secret();
```

`secrecy::Secret<T>` guarantees:
- `T: Zeroize` — memory is zeroed on drop
- `Debug` impl prints `"[REDACTED]"` — never leaks into logs
- No `Clone` or `Copy` — can't be accidentally duplicated

---

## Credential File Location

Resolved at runtime using `dirs::config_dir()`:

| Platform | Path |
|----------|------|
| Linux | `$XDG_CONFIG_HOME/ghost-io-api/credentials` or `~/.config/ghost-io-api/credentials` |
| macOS | `~/Library/Application Support/ghost-io-api/credentials` |
| Windows | `%AppData%\Roaming\ghost-io-api\credentials` |

The directory is created if it does not exist. File permissions should be set to `0600` (owner read/write only) on Unix systems.

---

## Encryption / Decryption Flow

### Write (store credentials)

```
1. Generate 16-byte random salt
2. Derive 32-byte key:  Argon2id(passphrase, salt) → key
3. Generate 24-byte random nonce
4. Serialize credentials to JSON bytes
5. Encrypt: XChaCha20-Poly1305(key, nonce, plaintext) → ciphertext
6. Write file: [salt | nonce | ciphertext]
7. Zeroize key bytes from memory
```

### Read (load credentials)

```
1. Read file → split into [salt (16)] [nonce (24)] [ciphertext (rest)]
2. Derive key: Argon2id(passphrase, salt) → key
3. Decrypt + verify MAC: XChaCha20-Poly1305(key, nonce, ciphertext) → plaintext
   → Returns error if MAC check fails (file tampered or wrong passphrase)
4. Deserialize JSON → GhostCredentials
5. Wrap strings in Secret<String>
6. Zeroize key bytes from memory
```

---

## Passphrase Sources (Priority Order)

```
1. GHOST_CREDENTIALS_PASSPHRASE env var   (CI, containers, automated deployments)
2. Interactive terminal prompt             (developer first-run setup)
```

The passphrase itself should be held in `Secret<String>` and zeroized after key derivation.

---

## What Is Never Stored

| Data | Stored? |
|------|---------|
| Passphrase | Never — only the derived key is used, then zeroed |
| Encryption key | Never — derived fresh on each load, zeroed after use |
| Salt + nonce | Yes (plaintext in file — not secret by design) |
| Credentials | Yes (encrypted ciphertext only) |

---

## Module Placement

```
src/
└── credentials/
    ├── mod.rs       # pub re-exports: GhostCredentials, load, save
    ├── crypto.rs    # argon2 KDF + xchacha20poly1305 encrypt/decrypt
    └── store.rs     # file path resolution (dirs), read/write, permission setting
```

---

## Security Properties Summary

| Property | Mechanism |
|----------|-----------|
| Encrypted at rest | XChaCha20-Poly1305 AEAD |
| Passphrase hardening | Argon2id (memory-hard KDF) |
| Tamper detection | Poly1305 MAC — decryption fails if file is modified |
| Memory safety | `secrecy::Secret<T>` + `zeroize` |
| No accidental logging | `Secret<T>` has `Debug = "[REDACTED]"` |
| Cross-platform | Pure Rust — no OS APIs, no daemons, no FFI |
| Nonce safety | 24-byte XChaCha nonce — collision probability negligible even with random generation |
