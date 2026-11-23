# Licencias de terceros (dependencias)

Este documento concentra las licencias de todas las dependencias de MathMaster,
incluidas las transitivas. Complementa [`CREDITS.md`](CREDITS.md) (créditos de
contenido, sonidos e iconos) y [`LICENSE`](LICENSE) (licencia del propio
proyecto).

## Licencia del proyecto

- **MathMaster** — `AGPL-3.0-or-later` — ver [`LICENSE`](LICENSE).

## Dependencias directas

Lista de las dependencias declaradas directamente en los `Cargo.toml` del
workspace, con su versión y licencia (según `cargo deny list`).

### Crate raíz (workspace)

| Dependencia | Versión | Licencia |
|-------------|---------|----------|
| serde | 1.0.229 | MIT |
| ron | 0.12.3 | MIT |
| thiserror | 2.0.21 | MIT |
| leptos | 0.8.23 | MIT |
| wasm-bindgen | 0.2.130 | MIT |
| js-sys | 0.3.107 | MIT |
| web-sys | 0.3.107 | MIT |
| console_error_panic_hook | 0.1.7 | MIT |

### `crates/core` (`anzan-core`)

| Dependencia | Versión | Licencia |
|-------------|---------|----------|
| thiserror | 2.0.21 | MIT |
| proptest | 1.12.0 | Apache-2.0 OR MIT *(dev-dependency)* |

### `crates/engine` (`anzan-engine`)

| Dependencia | Versión | Licencia |
|-------------|---------|----------|
| anzan-core | 0.1.0 | AGPL-3.0-or-later (workspace) |
| thiserror | 2.0.21 | MIT |
| proptest | 1.12.0 | Apache-2.0 OR MIT *(dev-dependency)* |

### `crates/content` (`anzan-content`)

| Dependencia | Versión | Licencia |
|-------------|---------|----------|
| anzan-core | 0.1.0 | AGPL-3.0-or-later (workspace) |
| serde | 1.0.229 | MIT |
| ron | 0.12.3 | MIT |
| thiserror | 2.0.21 | MIT |

### `crates/ui` (`anzan-ui`)

| Dependencia | Versión | Licencia |
|-------------|---------|----------|
| anzan-core | 0.1.0 | AGPL-3.0-or-later (workspace) |
| anzan-engine | 0.1.0 | AGPL-3.0-or-later (workspace) |
| anzan-content | 0.1.0 | AGPL-3.0-or-later (workspace) |
| leptos | 0.8.23 | MIT |
| wasm-bindgen | 0.2.130 | MIT |
| js-sys | 0.3.107 | MIT |
| web-sys | 0.3.107 | MIT |
| wasm-bindgen-futures | 0.4.80 | MIT |

### `crates/app` (`anzan-app`)

| Dependencia | Versión | Licencia |
|-------------|---------|----------|
| anzan-core | 0.1.0 | AGPL-3.0-or-later (workspace) |
| anzan-engine | 0.1.0 | AGPL-3.0-or-later (workspace) |
| anzan-content | 0.1.0 | AGPL-3.0-or-later (workspace) |
| anzan-ui | 0.1.0 | AGPL-3.0-or-later (workspace) |
| wasm-bindgen | 0.2.130 | MIT |
| web-sys | 0.3.107 | MIT |
| console_error_panic_hook | 0.1.7 | MIT |

### `crates/desktop` (`anzan-desktop`)

| Dependencia | Versión | Licencia |
|-------------|---------|----------|
| anzan-content | 0.1.0 | AGPL-3.0-or-later (workspace) |
| anzan-engine | 0.1.0 | AGPL-3.0-or-later (workspace) |
| cpal | 0.18.2 | Apache-2.0 |
| rtrb | 0.4.1 | MIT |
| serde | 1.0.229 | MIT |
| serde_json | 1.0.150 | MIT |
| tauri | 2.12.5 | Apache-2.0 OR MIT |
| tauri-build | 2.7.4 | Apache-2.0 OR MIT *(build-dependency)* |

### Herramientas y componentes de plataforma (no dependencias de Cargo)

| Componente | Versión | Licencia |
|------------|---------|----------|
| Rust toolchain (rustc, cargo) | 1.99.1 | MIT OR Apache-2.0 |
| wasm-bindgen CLI | 0.2.130 | MIT OR Apache-2.0 |
| Tauri CLI | 2.12.4 | Apache-2.0 OR MIT |
| cargo-deny | 0.22.0 | MIT OR Apache-2.0 |
| WebKit2GTK | 4.1 (sistema) | LGPL-2.1-or-later |
| GTK | 3.x (sistema) | LGPL-2.1-or-later |
| ALSA | sistema | LGPL-2.1-or-later |
| Web Audio API | implementación del navegador | Especificación W3C libre de regalías |

## Dependencias transitivas (resumen por licencia)

La lista completa de dependencias transitivas y sus licencias se genera con
`cargo deny list` (configurado en [`deny.toml`](deny.toml)). A continuación el
resumen de la última ejecución:

| Licencia | Cantidad |
|----------|----------|
| MIT | 449 |
| Apache-2.0 | 310 |
| AGPL-3.0-or-later | 6 |
| Zlib | 29 |
| MPL-2.0 | 7 |
| Unlicense | 6 |
| Unicode-3.0 | 19 |
| CC0-1.0 | 2 |
| BSD-3-Clause | 6 |
| BSD-2-Clause | 1 |
| Apache-2.0 WITH LLVM-exception | 4 |
| BSL-1.0 | 1 |
| LGPL-2.1-or-later | 2 |
| 0BSD | 1 |
| MIT-0 | 1 |

### Detalle por licencia

**AGPL-3.0-or-later (6):** `anzan-app`, `anzan-content`, `anzan-core`, `anzan-desktop`, `anzan-engine`, `anzan-ui`

**Apache-2.0 (310):** `adler2`, `alsa`, `anyhow`, `async-lock`, `async-once-cell`, `async-trait`, `atomic-waker`, `attribute-derive`, `autocfg`, `base64`, `bit-set`, `bit-vec`, `bitflags`, `block-buffer`, `bumpalo`, `bytemuck`, `camino`, `cargo-platform`, `cargo_toml`, `cc`, `cesu8`, `cfg-expr`, `cfg-if`, `codee`, `config`, `console_error_panic_hook`, `cookie`, `core-foundation`, `core-foundation-sys`, `core-graphics`, `core-graphics-types`, `coreaudio-rs`, `cpal`, `cpufeatures`, `crc32fast`, `crossbeam-channel`, `crossbeam-utils`, `crypto-common`, `ctor`, `dasp_sample`, `dbus`, `deranged`, `derive-where`, `digest`, `dirs`, `dirs-sys`, `dispatch2`, `displaydoc`, `dtoa`, `dunce`, `dyn-clone`, `either`, `embed_plist`, `equivalent`, `erased-serde`, `event-listener`, `event-listener-strategy`, `fastrand`, `fdeflate`, `field-offset`, `find-msvc-tools`, `flate2`, `fnv`, `foreign-types`, `foreign-types-macros`, `foreign-types-shared`, `form_urlencoded`, `futures`, `futures-channel`, `futures-core`, `futures-executor`, `futures-io`, `futures-macro`, `futures-sink`, `futures-task`, `futures-util`, `getrandom`, `glob`, `gloo-net`, `gloo-utils`, `guardian`, `hashbrown`, `heck`, `html5ever`, `http`, `httparse`, `ident_case`, `idna`, `idna_adapter`, `indexmap`, `interpolator`, `ipnet`, `itertools`, `itoa`, `jni`, `jni-macros`, `jni-sys`, `jni-sys-macros`, `js-sys`, `json-patch`, `jsonptr`, `keyboard-types`, `libc`, `libdbus-sys`, `link-section`, `lock_api`, `log`, `mach2`, `manyhow`, `manyhow-macros`, `markup5ever`, `mime`, `miniz_oxide`, `muda`, `ndk`, `ndk-context`, `ndk-sys`, `num-conv`, `num-derive`, `num-traits`, `num_enum`, `num_enum_derive`, `objc2-*`, `once_cell`, `parking`, `parking_lot`, `parking_lot_core`, `paste`, `pathdiff`, `percent-encoding`, `pin-project`, `pin-project-internal`, `pin-project-lite`, `pkg-config`, `png`, `powerfmt`, `prettyplease`, `proc-macro-crate`, `proc-macro-error`, `proc-macro-error-attr`, `proc-macro-utils`, `proc-macro2`, `proc-macro2-diagnostics`, `quote`, `r-efi`, `rand_core`, `raw-window-handle`, `regex`, `regex-automata`, `regex-syntax`, `reqwest`, `ron`, `rtrb`, `rustc-hash`, `rustc_version`, `rustversion`, `scopeguard`, `semver`, `send_wrapper`, `serde`, `serde-untagged`, `serde_core`, `serde_derive`, `serde_derive_internals`, `serde_json`, `serde_qs`, `serde_repr`, `serde_spanned`, `serde_with`, `serde_with_macros`, `serialize-to-javascript`, `serialize-to-javascript-impl`, `servo_arc`, `sha2`, `shlex`, `simd_cesu8`, `simdutf8`, `siphasher`, `smallvec`, `socket2`, `softbuffer`, `stable_deref_trait`, `static_vcruntime`, `string_cache`, `string_cache_codegen`, `swift-rs`, `syn`, `sync_wrapper`, `system-deps`, `tao`, `tao-macros`, `tauri`, `tauri-build`, `tauri-codegen`, `tauri-macros`, `tauri-runtime`, `tauri-runtime-wry`, `tauri-utils`, `tendril`, `thiserror`, `time`, `time-core`, `time-macros`, `toml`, `toml_datetime`, `toml_edit`, `toml_parser`, `toml_writer`, `typed-builder`, `typed-builder-macro`, `typeid`, `typenum`, `unicode-ident`, `unicode-segmentation`, `unicode-xid`, `url`, `utf8_iter`, `utf_types`, `uuid`, `version_check`, `wasi`, `wasip2`, `wasm-bindgen`, `wasm-bindgen-futures`, `wasm-bindgen-macro`, `wasm-bindgen-macro-support`, `wasm-bindgen-shared`, `wasm-streams`, `wasm_split_helpers`, `wasm_split_macros`, `web-sys`, `web-time`, `web_atoms`, `webkit2gtk`, `webkit2gtk-sys`, `webview2-com`, `webview2-com-macros`, `webview2-com-sys`, `winapi`, `windows`, `windows-core`, `windows-future`, `windows-implement`, `windows-interface`, `windows-link`, `windows-numerics`, `windows-result`, `windows-strings`, `windows-sys`, `windows-targets`, `windows-threading`, `windows-version`, `windows_aarch64_*`, `windows_i686_*`, `windows_x86_64_*`, `wit-bindgen`, `wry`, `x11`, `x11-dl`, `zmij`

**Apache-2.0 WITH LLVM-exception (4):** `target-lexicon`, `wasi`, `wasip2`, `wit-bindgen`

**BSD-2-Clause (1):** `mach2`

**BSD-3-Clause (6):** `alloc-no-stdlib`, `alloc-stdlib`, `brotli`, `brotli-decompressor`, `num_enum`, `num_enum_derive`

**BSL-1.0 (1):** `xxhash-rust`

**CC0-1.0 (2):** `base16`, `dunce`

**LGPL-2.1-or-later (2):** `r-efi` (5.3.0, 6.0.0)

**MIT (449):** `adler2`, `aho-corasick`, `alsa`, `alsa-sys`, `any_spawner`, `anyhow`, `async-lock`, `async-once-cell`, `async-trait`, `atk`, `atk-sys`, `atomic-waker`, `attribute-derive`, `attribute-derive-macro`, `autocfg`, `base64`, `bit-set`, `bit-vec`, `bitflags`, `block-buffer`, `block2`, `brotli`, `brotli-decompressor`, `bumpalo`, `bytemuck`, `byteorder`, `bytes`, `cairo-rs`, `cairo-sys-rs`, `camino`, `cargo-platform`, `cargo_metadata`, `cargo_toml`, `cc`, `cesu8`, `cfg-expr`, `cfg-if`, `codee`, `collection_literals`, `combine`, `config`, `console_error_panic_hook`, `const-str`, `const_str_slice_concat`, `convert_case`, `cookie`, `core-foundation`, `core-foundation-sys`, `core-graphics`, `core-graphics-types`, `coreaudio-rs`, `cpufeatures`, `crc32fast`, `crossbeam-channel`, `crossbeam-utils`, `crypto-common`, `ctor`, `darling`, `darling_core`, `darling_macro`, `dasp_sample`, `dbus`, `deranged`, `derive-where`, `derive_more`, `derive_more-impl`, `digest`, `dirs`, `dirs-sys`, `dispatch2`, `displaydoc`, `dlopen2`, `dlopen2_derive`, `dom_query`, `dpi`, `drain_filter_polyfill`, `dtoa`, `dyn-clone`, `either`, `either_of`, `embed-resource`, `embed_plist`, `equivalent`, `erased`, `erased-serde`, `event-listener`, `event-listener-strategy`, `fastrand`, `fdeflate`, `field-offset`, `find-msvc-tools`, `flate2`, `fnv`, `foreign-types`, `foreign-types-macros`, `foreign-types-shared`, `form_urlencoded`, `futures`, `futures-channel`, `futures-core`, `futures-executor`, `futures-io`, `futures-macro`, `futures-sink`, `futures-task`, `futures-util`, `gdk`, `gdk-pixbuf`, `gdk-pixbuf-sys`, `gdk-sys`, `gdkwayland-sys`, `gdkx11`, `gdkx11-sys`, `generic-array`, `getrandom`, `gio`, `gio-sys`, `glib`, `glib-macros`, `glib-sys`, `glob`, `gloo-net`, `gloo-utils`, `gobject-sys`, `gtk`, `gtk-sys`, `gtk3-macros`, `guardian`, `hashbrown`, `heck`, `html-escape`, `html5ever`, `http`, `http-body`, `http-body-util`, `httparse`, `hydration_context`, `hyper`, `hyper-util`, `ico`, `ident_case`, `idna`, `idna_adapter`, `indexmap`, `infer`, `interpolator`, `ipnet`, `itertools`, `itoa`, `javascriptcore-rs`, `javascriptcore-rs-sys`, `jni`, `jni-macros`, `jni-sys`, `jni-sys-macros`, `leptos`, `leptos_config`, `leptos_dom`, `leptos_hot_reload`, `leptos_macro`, `leptos_server`, `libc`, `libdbus-sys`, `libredox`, `link-section`, `lock_api`, `log`, `mach2`, `manyhow`, `manyhow-macros`, `markup5ever`, `memchr`, `memoffset`, `mime`, `miniz_oxide`, `mio`, `muda`, `ndk`, `ndk-context`, `ndk-sys`, `new_debug_unreachable`, `next_tuple`, `num-conv`, `num-derive`, `num-traits`, `num_enum`, `num_enum_derive`, `objc2`, `objc2-*`, `oco_ref`, `once_cell`, `or_poisoned`, `pango`, `pango-sys`, `parking`, `parking_lot`, `parking_lot_core`, `paste`, `pathdiff`, `percent-encoding`, `phf`, `pin-project`, `pin-project-internal`, `pin-project-lite`, `pkg-config`, `plist`, `png`, `powerfmt`, `precomputed-hash`, `prettyplease`, `proc-macro-crate`, `proc-macro-error`, `proc-macro-error-attr`, `proc-macro-utils`, `proc-macro2`, `proc-macro2-diagnostics`, `quick-xml`, `quote`, `quote-use`, `quote-use-macros`, `r-efi`, `rand_core`, `raw-window-handle`, `reactive_graph`, `reactive_stores`, `reactive_stores_macro`, `redox_syscall`, `redox_users`, `regex`, `regex-automata`, `regex-syntax`, `reqwest`, `ron`, `rstml`, `rustc-hash`, `rustc_version`, `rustversion`, `same-file`, `schemars`, `schemars_derive`, `scopeguard`, `semver`, `send_wrapper`, `serde`, `serde-untagged`, `serde_core`, `serde_derive`, `serde_derive_internals`, `serde_json`, `serde_qs`, `serde_repr`, `serde_spanned`, `serde_with`, `serde_with_macros`, `server_fn`, `server_fn_macro`, `server_fn_macro_default`, `servo_arc`, `sha2`, `shlex`, `simd-adler32`, `simd_cesu8`, `simdutf8`, `siphasher`, `slab`, `smallvec`, `socket2`, `softbuffer`, `soup3`, `soup3-sys`, `stable_deref_trait`, `static_vcruntime`, `string_cache`, `string_cache_codegen`, `strsim`, `swift-rs`, `syn`, `synstructure`, `system-deps`, `tachys`, `tao-macros`, `tauri`, `tauri-build`, `tauri-codegen`, `tauri-macros`, `tauri-runtime`, `tauri-runtime-wry`, `tauri-utils`, `tauri-winres`, `tendril`, `thiserror`, `throw_error`, `time`, `time-core`, `time-macros`, `tokio`, `tokio-util`, `toml`, `toml_datetime`, `toml_edit`, `toml_parser`, `toml_writer`, `tower`, `tower-http`, `tower-layer`, `tower-service`, `tracing`, `tracing-core`, `try-lock`, `typed-builder`, `typed-builder-macro`, `typeid`, `typenum`, `unicode-ident`, `unicode-segmentation`, `unicode-xid`, `url`, `urlpattern`, `utf8_iter`, `utf_types`, `uuid`, `version-compare`, `version_check`, `vswhom`, `vswhom-sys`, `walkdir`, `want`, `wasi`, `wasip2`, `wasm-bindgen`, `wasm-bindgen-futures`, `wasm-bindgen-macro`, `wasm-bindgen-macro-support`, `wasm-bindgen-shared`, `wasm-streams`, `wasm_split_helpers`, `wasm_split_macros`, `web-sys`, `web-time`, `web_atoms`, `webkit2gtk`, `webkit2gtk-sys`, `webview2-com`, `webview2-com-macros`, `webview2-com-sys`, `winapi`, `winapi-i686-pc-windows-gnu`, `winapi-util`, `winapi-x86_64-pc-windows-gnu`, `window-vibrancy`, `windows`, `windows-collections`, `windows-core`, `windows-future`, `windows-implement`, `windows-interface`, `windows-link`, `windows-numerics`, `windows-result`, `windows-strings`, `windows-sys`, `windows-targets`, `windows-threading`, `windows-version`, `windows_aarch64_*`, `windows_i686_*`, `windows_x86_64_*`, `winnow`, `winreg`, `wit-bindgen`, `wry`, `x11`, `x11-dl`, `zmij`

**MIT-0 (1):** `dunce`

**MPL-2.0 (7):** `cssparser`, `cssparser-macros`, `dtoa-short`, `option-ext`, `selectors`

**Unicode-3.0 (19):** `icu_collections`, `icu_locale_core`, `icu_normalizer`, `icu_normalizer_data`, `icu_properties`, `icu_properties_data`, `icu_provider`, `litemap`, `potential_utf`, `tinystr`, `unicode-ident`, `writeable`, `yoke`, `yoke-derive`, `zerofrom`, `zerofrom-derive`, `zerotrie`, `zerovec`, `zerovec-derive`

**Unlicense (6):** `aho-corasick`, `byteorder`, `memchr`, `same-file`, `walkdir`, `winapi-util`

**Zlib (29):** `bytemuck`, `const_format`, `const_format_proc_macros`, `dispatch2`, `foldhash`, `konst`, `konst_macro_rules`, `miniz_oxide`, `objc2-*`, `raw-window-handle`, `slotmap`, `static_vcruntime`

**0BSD (1):** `adler2`

## Cómo regenerar esta lista

```bash
# Instalar cargo-deny si no está disponible
cargo install cargo-deny

# Generar la lista completa de dependencias y licencias
cargo deny list

# Auditar licencias permitidas (según deny.toml)
cargo deny check licenses
```

## Notas

- Las dependencias del propio proyecto (`anzan-*`) están bajo `AGPL-3.0-or-later`, la misma licencia de MathMaster.
- `deny.toml` documenta las licencias permitidas y las advertencias de `cargo-deny`.
- Algunas dependencias son del sistema (GTK, WebKit2GTK, ALSA) y no están en `Cargo.lock`; se listan por separado arriba.
