# Changelog

## \[2.0.0-alpha.2]

- [`c81dff2`](https://github.com/tauri-apps/plugins-workspace/commit/c81dff292afc9e9f52470a8d9e34b5a00c14b3a0)([#440](https://github.com/tauri-apps/plugins-workspace/pull/440)) Revert [7d71ad4e5](https://github.com/tauri-apps/plugins-workspace/commit/7d71ad4e587bcf47ea34645f5b226945e487b765) which added a default sound for notifications on Windows. This introduced inconsistency with other platforms that has silent notifications by default. In the upcoming releases, we will add support for modifying the notification sound across all platforms.

## \[2.0.0-alpha.1]

- [`d8b4aca`](https://github.com/tauri-apps/plugins-workspace/commit/d8b4aca69f628b170804ecb982e2c319d026ef47)([#414](https://github.com/tauri-apps/plugins-workspace/pull/414)) Use `window.__TAURI_INVOKE__` instead of `window.__TAURI__` in init.js, fixes usage in apps without `withGlobalTauri` enabled.
- [`7d71ad4`](https://github.com/tauri-apps/plugins-workspace/commit/7d71ad4e587bcf47ea34645f5b226945e487b765) Play a default sound when showing a notification on Windows.

## \[2.0.0-alpha.0]

- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
