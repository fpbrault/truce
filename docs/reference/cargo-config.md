# `.cargo/config.toml` reference (build-environment vars)

Per-developer credentials and machine-specific paths live in
your project's `.cargo/config.toml` `[env]` table — gitignored,
machine-local, never in `truce.toml`.

Cargo injects everything in `[env]` into the environment of any
subcommand it spawns, so values you set here are visible to
`cargo truce ...` automatically. Setting the same variable in
your shell also works; `[env]` table values take precedence
when present (or you can opt into shell precedence with
`force = false`, see Cargo's docs).

```toml
# .cargo/config.toml at the project root.
[env]
TRUCE_SIGNING_IDENTITY = "Developer ID Application: Your Name (TEAMID)"
AAX_SDK_PATH           = "/Users/you/aax-sdk-2-9-0"
```

## macOS code signing

| Variable | Purpose |
|----------|---------|
| `TRUCE_SIGNING_IDENTITY` | `codesign -s` identity for plug-in bundles. Full `"Developer ID Application: Name (TEAMID)"` or `"-"` for ad-hoc (the default). Required for AU v3 — Apple won't load an appex bundle without a real Developer ID. |
| `TRUCE_INSTALLER_SIGNING_IDENTITY` | `productbuild --sign` identity for `.pkg` installers. Typically `"Developer ID Installer: Name (TEAMID)"`. Unset → installer is unsigned. |

Set both for a release build; ad-hoc (`-`, the default) is fine
for local install + DAW load testing of CLAP / VST3 / AU v2.

## macOS notarization

Used by `cargo truce package` when `[macos.packaging].notarize = true`
in `truce.toml`. Two credential paths — the keychain profile is
preferred, explicit credentials are the fallback.

| Variable | Purpose |
|----------|---------|
| `TRUCE_NOTARY_PROFILE` | Keychain profile name for `xcrun notarytool`. Default `TRUCE_NOTARY`. Create with `xcrun notarytool store-credentials TRUCE_NOTARY`. |
| `APPLE_ID` | Apple ID email — fallback path when no keychain profile exists. |
| `TEAM_ID` | Apple Developer Team ID — pair with `APPLE_ID`. |
| `APP_SPECIFIC_PASSWORD` | App-specific password from appleid.apple.com — pair with `APPLE_ID` + `TEAM_ID`. Never put this in `truce.toml`. |

## AAX SDK

| Variable | Purpose |
|----------|---------|
| `AAX_SDK_PATH` | Absolute path to the AAX SDK root (the directory containing `Libs/`, `Interfaces/`, etc.). Required to build the `aax` format. macOS / Windows; Linux has no AAX. |

## Windows Authenticode signing

Pick one of three credential sources. First non-empty source
wins, in the order Azure → cert thumbprint → `.pfx`. Setting
none of these silently produces unsigned binaries (with a
warning at the top of `cargo truce package`).

### Azure Trusted Signing

| Variable | Purpose |
|----------|---------|
| `TRUCE_AZURE_ACCOUNT` | Azure Trusted Signing account name. |
| `TRUCE_AZURE_PROFILE` | Certificate profile name within the account. |
| `TRUCE_AZURE_DLIB` | Override path to `Azure.CodeSigning.Dlib.dll`. Defaults to `C:\Program Files\Microsoft Trusted Signing Client\bin\x64\Azure.CodeSigning.Dlib.dll`. |

Azure's own auth (tenant ID, client ID / secret) is read from
its standard env vars by the Azure SDK at signing time —
`AZURE_TENANT_ID`, `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`.
Set those alongside the `TRUCE_AZURE_*` vars above.

### Certificate thumbprint

| Variable | Purpose |
|----------|---------|
| `TRUCE_CERT_SHA1` | SHA1 thumbprint of a cert already in the Windows cert store. |
| `TRUCE_CERT_STORE` | Cert store name. Defaults to `My`. |

### `.pfx` file

| Variable | Purpose |
|----------|---------|
| `TRUCE_PFX_PATH` | Path to a `.pfx` file. |
| `TRUCE_PFX_PASSWORD` | Password for the `.pfx`. Never put this in `truce.toml`. |

### Common Windows signing

| Variable | Purpose |
|----------|---------|
| `TRUCE_TIMESTAMP_URL` | RFC 3161 timestamp server. Defaults to `http://timestamp.digicert.com`. |

## PACE / iLok signing (AAX retail)

| Variable | Purpose |
|----------|---------|
| `PACE_ACCOUNT` | iLok account name for `wraptool`. Required for retail Pro Tools releases. |
| `PACE_SIGN_ID` | PACE-issued signing ID. Pair with `PACE_ACCOUNT`. |

Without these, `cargo truce package` skips PACE wrapping. Local
testing in Pro Tools Developer with a dev iLok license doesn't
need them.

## Validators (`cargo truce validate`)

Truce locates each validator on `$PATH` by default; these
overrides point at non-`PATH` installs (Mac `.app` bundles,
sibling source checkouts).

| Variable | Purpose |
|----------|---------|
| `PLUGINVAL` | Override path to `pluginval` (or `pluginval.app/Contents/MacOS/pluginval`). |
| `CLAP_VALIDATOR` | Override path to `clap-validator`. |

## Build / toolchain

| Variable | Purpose |
|----------|---------|
| `MACOSX_DEPLOYMENT_TARGET` | Minimum macOS version for emitted bundles. Defaults to `11.0`. |
| `TRUCE_DISABLE_SCCACHE` | Set to `1` to disable the auto-`sccache` wrapper truce uses to speed up rebuilds. Useful when sccache misbehaves on a particular machine. |

## Hot-reload

These are read by the shell binary (the `truce::plugin!` macro
emits the read), not by `cargo truce`. They're listed here for
completeness because the developer sets them the same way.

| Variable | Purpose |
|----------|---------|
| `TRUCE_LOGIC_PATH` | Override path to the hot-reload logic dylib. Bypasses the sidecar file `cargo truce install --shell` writes. Useful for swapping between dev / release builds without re-running install. |

## What's NOT here

These don't exist as truce-controlled options. They're read by
underlying tools that truce orchestrates:

- `PATH`, `HOME`, `APPDATA`, `LOCALAPPDATA`, `USERPROFILE` —
  consumed by every CLI tool; truce just respects them.
- `RUSTC_WRAPPER`, `RUSTC_WORKSPACE_WRAPPER` — Cargo's wrapper
  hooks. Truce sets `RUSTC_WRAPPER=sccache` automatically when
  sccache is on `$PATH` and `TRUCE_DISABLE_SCCACHE` is unset.
- `AZURE_TENANT_ID`, `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET` —
  read by the Azure SDK / Microsoft Trusted Signing Client at
  signing time, not by truce. Set them alongside the
  `TRUCE_AZURE_*` vars when you use Azure signing.

## Example

A tracked-but-comment-only template ships in scaffolded
projects at `.cargo/config.toml`. A populated example for a
real release setup looks like:

```toml
# .cargo/config.toml — gitignored.
[env]
# macOS code signing
TRUCE_SIGNING_IDENTITY           = "Developer ID Application: Acme Audio, LLC (TEAM123)"
TRUCE_INSTALLER_SIGNING_IDENTITY = "Developer ID Installer: Acme Audio, LLC (TEAM123)"

# macOS notarization (keychain profile preferred; xcrun notarytool store-credentials TRUCE_NOTARY)

# AAX SDK
AAX_SDK_PATH = "/Users/you/aax-sdk-2-9-0"

# Windows Authenticode (Azure Trusted Signing)
TRUCE_AZURE_ACCOUNT = "AcmeSigning"
TRUCE_AZURE_PROFILE = "AcmeRelease"

# PACE / iLok (retail Pro Tools)
PACE_ACCOUNT = "acme-ilok-account"
PACE_SIGN_ID = "..."
```

Companion file: [`truce-toml.md`](truce-toml.md) for the
project-level config that does live in the tracked file.
