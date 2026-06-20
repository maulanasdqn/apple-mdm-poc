# rust-apple-mdm

A custom Apple **MDM (Mobile Device Management)** server in Rust, built to enroll
an iPhone and lock it down (restrictions, remote lock, query, erase).

It follows a clean-architecture Cargo workspace: `domain → application →
infrastructure → presentation`, with Axum 0.8, SQLite (sqlx), OpenSSL for
certificate/CMS signing, and APNs behind a swappable port.

> Status: **scaffold**. Enrollment-profile generation, device check-in, the
> command queue, and lockdown commands are implemented and verified end-to-end
> against a simulated device. SCEP issuance and the real APNs client are stubbed
> behind ports/feature flags (see [Limitations](#limitations)).

## Layout

```
.config/        shared config, logging, db pool        (crate: mdm-config)
.migrations/    sqlx migrations                          (crate: mdm-migrations)
apps/gateway/   binary entrypoint / composition root     (crate: gateway)
apps/mdm/       the MDM module, four clean-arch layers    (crate: mdm)
  domain/         entities + ports (traits), no framework deps
  application/    use cases (enroll, checkin, poll, enqueue)
  infrastructure/ adapters: sqlite, apns (noop/a2), cert (openssl CA + CMS)
  presentation/   axum router, handlers, DTOs, state
```

Dependency flow is enforced by module imports: `presentation → application →
domain`; `infrastructure` implements `domain::ports` and is injected at the
composition root in `apps/gateway/src/main.rs`.

## Run

```bash
cp .env.example .env          # adjust BASE_URL / MDM_TOPIC as needed
cargo run -p gateway
```

The server binds `SERVER_ADDR` (default `0.0.0.0:8080`), runs migrations against
`DATABASE_URL` (SQLite), and generates an ephemeral in-memory dev CA if no cert
files are present.

## Endpoints

| Method     | Path                      | Purpose                                            |
|------------|---------------------------|----------------------------------------------------|
| GET        | `/healthz`, `/readyz`     | Liveness / readiness                               |
| GET/POST   | `/enroll`                 | CMS-signed `.mobileconfig` enrollment profile      |
| PUT/POST   | `/checkin`                | Check-in: Authenticate / TokenUpdate / CheckOut    |
| PUT/POST   | `/server`                 | Command poll: returns next queued command          |
| GET/POST   | `/scep`                   | SCEP `GetCACert` / `GetCACaps`                     |
| POST       | `/scep/issue`             | Issue a device cert from a PKCS#10 CSR (PEM/DER)    |
| GET        | `/admin/devices`          | List enrolled devices (JSON)                       |
| POST       | `/admin/commands/{udid}`  | Enqueue a command (dev/automation)                 |

### Enqueue a lockdown command

```bash
# Remote-lock with a PIN
curl -X POST localhost:8080/admin/commands/<UDID> \
  -H 'Content-Type: application/json' \
  -d '{"type":"DeviceLock","pin":"123456","message":"Locked by MDM"}'

# Apply restrictions (compiled into an InstallProfile carrying
# com.apple.applicationaccess)
curl -X POST localhost:8080/admin/commands/<UDID> \
  -H 'Content-Type: application/json' \
  -d '{"type":"Restrictions","allow_camera":false,"allow_app_installation":false,"allow_safari":false}'
```

## How lockdown actually reaches a device

1. Device installs the signed enrollment profile from `/enroll`. The profile
   embeds a per-device **PKCS#12 identity** (CA-signed key + cert) as a
   `com.apple.security.pkcs12` payload, referenced by the MDM payload's
   `IdentityCertificateUUID` — so the device gets its mutual-TLS identity without
   a SCEP round-trip. The device then becomes managed (`Authenticate` →
   `TokenUpdate` to `/checkin`).
2. Server enqueues a command and sends an APNs **wake-up** push.
3. Device polls `/server`; the server returns the next command plist.
4. Device executes it and POSTs `Acknowledged` back.

"Restrictions" is not a top-level command — on iOS it is enforced by installing
a `com.apple.applicationaccess` configuration profile via `InstallProfile`.

## Limitations (read before testing on hardware)

These are **Apple platform** prerequisites, not code gaps — each is stubbed so
the scaffold runs locally with zero credentials:

- **The iOS Simulator cannot be MDM-managed.** It can't enroll in MDM, has no
  real APNs, no supervision, and `simctl` has no profile/MDM support. Lockdown
  must be tested on a **real iPhone**.
- **Supervision** (via Apple Configurator or Automated Device Enrollment / ABM)
  is required for silent install and most *restriction enforcement*.
  `DeviceLock` / `EraseDevice` work even unsupervised.
- **APNs vendor MDM push certificate** (Apple MDM Vendor program) is required to
  wake devices. Default is a no-op push that logs; build with
  `--features apns-real` and set `APNS_MODE=real` + `APNS_CERT_PATH` to deliver.
- **Device identity**: provisioned via an embedded **PKCS#12** in the enrollment
  profile (`com.apple.security.pkcs12`), generated and CA-signed server-side. No
  SCEP round-trip needed. Tradeoff vs SCEP: the private key is generated on the
  server rather than on-device. The SCEP issuance core (`POST /scep/issue`,
  `GetCACert`/`GetCACaps`) is also available; the full SCEP `PKIOperation`
  transport is intentionally not implemented (see `infrastructure/cert/scep.rs`).
- **Public HTTPS URL**: devices must reach `BASE_URL` over TLS — locally use an
  ngrok / cloudflared tunnel.

### To lock down a real iPhone

1. Get an Apple MDM vendor APNs push certificate; set `MDM_TOPIC` + APNs config.
2. Supervise the test iPhone with Apple Configurator (or enroll via ABM/ADE).
3. Expose the server over HTTPS (tunnel) and set `BASE_URL`.
4. Install `/enroll` on the device; confirm it appears in `/admin/devices`.
5. `POST /admin/commands/{udid}` a `DeviceLock` and confirm the device locks.

## Tests

```bash
cargo test --workspace
```

Covers command/check-in plist round-trips and CMS profile signing.
