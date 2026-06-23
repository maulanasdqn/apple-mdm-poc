# rust-apple-mdm

A custom Apple **MDM (Mobile Device Management)** server in Rust, built to enroll
an iPhone and lock it down (restrictions, remote lock, query, erase).

It speaks the Apple MDM protocol: it serves a signed enrollment profile, handles
device check-in, queues commands per device, and wakes devices over APNs to pull
those commands. Built with Axum 0.8, SQLite (sqlx), and OpenSSL for
certificate / CMS signing.

> Status: **scaffold**. Enrollment-profile generation, device check-in, the
> command queue, and lockdown commands are implemented and verified end-to-end
> against a simulated device. SCEP issuance and the real APNs client are stubbed
> behind ports/feature flags (see [Limitations](#limitations)).

## Layout

A Cargo workspace:

```
.config/        shared config, logging, db pool   (crate: mdm-config)
.migrations/    sqlx migrations                    (crate: mdm-migrations)
apps/gateway/   binary entrypoint, starts the server (crate: gateway)
apps/mdm/       the MDM server library             (crate: mdm)
apps/worker/    WASM build for Cloudflare Workers  (crate: mdm-worker)
```

`apps/worker/` is a separate, self-contained WebAssembly port that runs on
Cloudflare Workers with D1 storage — the native openssl/sqlx/tokio stack can't
target wasm32. It's the deployed version (see below).

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

## Deployed: Cloudflare Workers (`mdm.stynx.app`)

The `apps/worker/` crate is deployed as a WebAssembly Worker backed by D1.
It serves an unsigned enrollment profile over Cloudflare's TLS and embeds one
OpenSSL-built PKCS#12 device identity (shared across devices — fine for
personal lockdown, not multi-tenant).

```bash
cd apps/worker
CLOUDFLARE_ACCOUNT_ID=<acct> npx wrangler deploy   # build (worker-build) + deploy
npx wrangler tail mdm                              # live logs
npx wrangler d1 execute mdm --remote --file=schema.sql   # (re)apply schema
```

Build notes: needs `worker` crate >= 0.8 with `worker-build` >= 0.8.5, and the
release profile must **not** set `strip = true` (stripping removes the wasm
`target_features` section wasm-bindgen needs).

### Command reference

All commands: `POST /admin/commands/<UDID>` with a JSON body. The `type`
discriminates. "Supervised?" marks commands that only take effect on a
supervised device (Apple Configurator / ADE); the rest work on any enrolled
device.

| `type`              | Body fields                                                                 | Effect                                  | Supervised? |
|---------------------|-----------------------------------------------------------------------------|-----------------------------------------|:-----------:|
| `DeviceLock`        | `message?`, `phone_number?`, `pin?` (PIN = macOS firmware only)              | Lock the screen                         | no          |
| `EraseDevice`       | `pin?`                                                                       | Wipe the device                         | no          |
| `DeviceInformation` | `queries?` (defaults to name/OS/product/serial)                             | Query device facts                      | no          |
| `EnableLostMode`    | `message?`, `phone_number?`, `footnote?`                                     | Lock to a message; user can't disable   | **yes**     |
| `DisableLostMode`   | —                                                                           | Release Lost Mode                       | **yes**     |
| `PlayLostModeSound` | —                                                                           | Ring while in Lost Mode                 | **yes**     |
| `DeviceLocation`    | —                                                                           | Return location (in Lost Mode)          | **yes**     |
| `Restrictions`      | any of `allow_camera`, `allow_safari`, `allow_app_installation`, `allow_app_removal`, `allow_screenshot`, `allow_erase_content_and_settings`, `allow_account_modification`, `allow_ui_configuration_profile_installation`, `allow_activation_lock`, `force_automatic_date_and_time` | Install a `com.apple.applicationaccess` profile | **yes** |
| `Lockdown`          | —                                                                           | Strict preset: blocks factory reset, account changes, profile/app changes | **yes** |
| `RemoveProfile`     | `identifier`                                                                | Remove a managed profile                | no          |

```bash
# Lock the screen
curl -X POST https://mdm.stynx.app/admin/commands/<UDID> \
  -H 'Content-Type: application/json' \
  -d '{"type":"DeviceLock","message":"Locked by Rust MDM"}'

# Borrower lockout on a SUPERVISED device: prevent escape, then Lost-Mode lock
curl -X POST https://mdm.stynx.app/admin/commands/<UDID> \
  -H 'Content-Type: application/json' -d '{"type":"Lockdown"}'
curl -X POST https://mdm.stynx.app/admin/commands/<UDID> \
  -H 'Content-Type: application/json' \
  -d '{"type":"EnableLostMode","message":"This device is locked. Contact owner.","phone_number":"+62...","footnote":"Property of owner"}'
```

> **Supervision** (which erases the device) is required for Lost Mode and all
> restrictions — Apple ignores them on an unsupervised device, where the holder
> can also just remove the management profile. **APNs is no-op**, so commands
> are delivered on the device's own poll cadence, not via push.

### Local dev (native gateway)

```bash
curl -X POST localhost:8080/admin/commands/<UDID> \
  -H 'Content-Type: application/json' \
  -d '{"type":"DeviceLock","pin":"123456","message":"Locked by MDM"}'
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
