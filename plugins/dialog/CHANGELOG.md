# Changelog

## \[2.0.0-beta.6]

- [`326df688`](https://github.com/tauri-apps/plugins-workspace/commit/326df6883998d416fc0837583ed972854628bb52)([#1236](https://github.com/tauri-apps/plugins-workspace/pull/1236)) Fixes command argument parsing on iOS.

### Dependencies

- Upgraded to `fs@2.0.0-beta.6`

## \[2.0.0-beta.5]

- [`bb51a41`](https://github.com/tauri-apps/plugins-workspace/commit/bb51a41d67ebf989e8aedf10c4b1a7f9514d1bdf)([#1168](https://github.com/tauri-apps/plugins-workspace/pull/1168)) **Breaking Change:** All apis that return paths to the frontend will now remove the `\\?\` UNC prefix on Windows.

### Dependencies

- Upgraded to `fs@2.0.0-beta.5`

## \[2.0.0-beta.4]

- [`4cd8112`](https://github.com/tauri-apps/plugins-workspace/commit/4cd81126fdf25e1847546f8fdbd924aa4bfeabb5)([#1056](https://github.com/tauri-apps/plugins-workspace/pull/1056)) Fixed an issue where dialogs on android would return the Content URI instead of the file path

### Dependencies

- Upgraded to `fs@2.0.0-beta.4`

## \[2.0.0-beta.3]

- [`35ea595`](https://github.com/tauri-apps/plugins-workspace/commit/35ea5956d060f0bdafd140f2541c607bb811805b)([#1073](https://github.com/tauri-apps/plugins-workspace/pull/1073)) Fixed an issue where the dialog apis panicked when they were called with no application windows open.
- [`a04ea2f`](https://github.com/tauri-apps/plugins-workspace/commit/a04ea2f38294d5a3987578283badc8eec87a7752)([#1071](https://github.com/tauri-apps/plugins-workspace/pull/1071)) The global API script is now only added to the binary when the `withGlobalTauri` config is true.

### Dependencies

- Upgraded to `fs@2.0.0-beta.3`

## \[2.0.0-beta.2]

- [`aa25c91`](https://github.com/tauri-apps/plugins-workspace/commit/aa25c91bb01e957872fb2b606a3acaf2f2c4ef3a)([#978](https://github.com/tauri-apps/plugins-workspace/pull/978)) Allow configuring `canCreateDirectories` for open and save dialogs on macOS, if not configured, it will be set to `true` by default.
- [`99bea25`](https://github.com/tauri-apps/plugins-workspace/commit/99bea2559c2c0648c2519c50a18cd124dacef57b)([#1005](https://github.com/tauri-apps/plugins-workspace/pull/1005)) Update to tauri beta.8.

## \[2.0.0-beta.1]

- [`569defb`](https://github.com/tauri-apps/plugins-workspace/commit/569defbe9492e38938554bb7bdc1be9151456d21) Update to tauri beta.4.

## \[2.0.0-beta.0]

- [`d198c01`](https://github.com/tauri-apps/plugins-workspace/commit/d198c014863ee260cb0de88a14b7fc4356ef7474)([#862](https://github.com/tauri-apps/plugins-workspace/pull/862)) Update to tauri beta.

## \[2.0.0-alpha.7]

### Dependencies

- Upgraded to `fs@2.0.0-alpha.7`

## \[2.0.0-alpha.5]

- [`387c2f9`](https://github.com/tauri-apps/plugins-workspace/commit/387c2f9e0ce4c75c07ffa3fd76391a25b58f5daf)([#802](https://github.com/tauri-apps/plugins-workspace/pull/802)) Update to @tauri-apps/api v2.0.0-alpha.13.

## \[2.0.0-alpha.4]

- [`387c2f9`](https://github.com/tauri-apps/plugins-workspace/commit/387c2f9e0ce4c75c07ffa3fd76391a25b58f5daf)([#802](https://github.com/tauri-apps/plugins-workspace/pull/802)) Update to @tauri-apps/api v2.0.0-alpha.12.
- [`2e2c0a1`](https://github.com/tauri-apps/plugins-workspace/commit/2e2c0a1b958fcbc7de855ef1d305b11855f2eb8e)([#769](https://github.com/tauri-apps/plugins-workspace/pull/769)) Fix incorrect result for dialog messages.

## \[2.0.0-alpha.3]

- [`e438e0a`](https://github.com/tauri-apps/plugins-workspace/commit/e438e0a62d4b430a5159f05f13ecd397dd891a0d)([#676](https://github.com/tauri-apps/plugins-workspace/pull/676)) Update to @tauri-apps/api v2.0.0-alpha.11.

## \[2.0.0-alpha.2]

- [`5c13736`](https://github.com/tauri-apps/plugins-workspace/commit/5c137365c60790e8d4037d449e8237aa3fffdab0)([#673](https://github.com/tauri-apps/plugins-workspace/pull/673)) Update to @tauri-apps/api v2.0.0-alpha.9.

## \[2.0.0-alpha.2]

- [`4e2cef9`](https://github.com/tauri-apps/plugins-workspace/commit/4e2cef9b702bbbb9cf4ee17de50791cb21f1b2a4)([#593](https://github.com/tauri-apps/plugins-workspace/pull/593)) Update to alpha.12.

### Dependencies

- Upgraded to `fs@2.0.0-alpha.2`

## \[2.0.0-alpha.1]

- [`d74fc0a`](https://github.com/tauri-apps/plugins-workspace/commit/d74fc0a097996e90a37be8f57d50b7d1f6ca616f)([#555](https://github.com/tauri-apps/plugins-workspace/pull/555)) Update to alpha.11.

## \[2.0.0-alpha.0]

- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  d6e80b)([#545](https://github.com/tauri-apps/plugins-workspace/pull/545)) Fixes docs.rs build by enabling the `tauri/dox` feature flag.
- [`d74fc0a`](https://github.com/tauri-apps/plugins-workspace/commit/d74fc0a097996e90a37be8f57d50b7d1f6ca616f)([#555](https://github.com/tauri-apps/plugins-workspace/pull/555)) Update to alpha.11.

### Dependencies

- Upgraded to `fs@2.0.0-alpha.1`

## \[2.0.0-alpha.0]

- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  \`

## \[2.0.0-alpha.0]

- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  pull/371)) First v2 alpha release!
  ri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  \`

## \[2.0.0-alpha.0]

- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  pull/371)) First v2 alpha release!
  hub.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  pull/371)) First v2 alpha release!
  ri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  \`

## \[2.0.0-alpha.0]

- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  pull/371)) First v2 alpha release!
  alpha release!
  pull/371)) First v2 alpha release!
  ri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  \`

## \[2.0.0-alpha.0]

- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  pull/371)) First v2 alpha release!
  kspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  pull/371)) First v2 alpha release!
  71]\(https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  pull/371)) First v2 alpha release!
  kspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  pull/371)) First v2 alpha release!
