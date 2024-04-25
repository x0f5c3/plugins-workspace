![plugin-autostart](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/autostart/banner.png)

Automatically launch your application at startup. Supports Windows, Mac (via AppleScript or Launch Agent), and Linux.

## Install

_This plugin requires a Rust version of at least **1.75**_

There are three general methods of installation that we can recommend.

1. Use crates.io and npm (easiest, and requires you to trust that our publishing pipeline worked)
2. Pull sources directly from Github using git tags / revision hashes (most secure)
3. Git submodule install this repo in your tauri project and then use file protocol to ingest the source (most secure, but inconvenient to use)

Install the Core plugin by adding the following to your `Cargo.toml` file:

`src-tauri/Cargo.toml`

```toml
[dependencies]
tauri-plugin-autostart = "2.0.0-beta"
# alternatively with Git:
tauri-plugin-autostart = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

> Note: Since most JavaScript package managers are unable to install packages from git monorepos we provide read-only mirrors of each plugin. This makes installation option 2 more ergonomic to use.

```sh
pnpm add @tauri-apps/plugin-autostart
# or
npm add @tauri-apps/plugin-autostart
# or
yarn add @tauri-apps/plugin-autostart

# alternatively with Git:
pnpm add https://github.com/tauri-apps/tauri-plugin-autostart#v2
# or
npm add https://github.com/tauri-apps/tauri-plugin-autostart#v2
# or
yarn add https://github.com/tauri-apps/tauri-plugin-autostart#v2
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/main.rs`

```rust
use tauri_plugin_autostart::MacosLauncher;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--flag1", "--flag2"]) /* arbitrary number of args to pass to your app */))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import { enable, isEnabled, disable } from "@tauri-apps/plugin-autostart";

await enable();

console.log(`registered for autostart? ${await isEnabled()}`);

disable();
```

## Contributing

PRs accepted. Please make sure to read the Contributing Guide before making a pull request.

## Partners

<table>
  <tbody>
    <tr>
      <td align="center" valign="middle">
        <a href="https://crabnebula.dev" target="_blank">
          <img src="https://github.com/tauri-apps/plugins-workspace/raw/v2/.github/sponsors/crabnebula.svg" alt="CrabNebula" width="283">
        </a>
      </td>
    </tr>
  </tbody>
</table>

For the complete list of sponsors please visit our [website](https://tauri.app#sponsors) and [Open Collective](https://opencollective.com/tauri).

## License

Code: (c) 2015 - Present - The Tauri Programme within The Commons Conservancy.

MIT or MIT/Apache 2.0 where applicable.
