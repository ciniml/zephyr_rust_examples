# Zephyr Rust Examples with nRF Connect SDK

## Setup

* See NCS setup instruction https://docs.nordicsemi.com/bundle/ncs-latest/page/nrf/installation/install_ncs.html

### Install NCS toolchain

```
nrfutil sdk-manager toolchain install --ncs-version v3.1.0-preview1
```

### Install NCS

```
nrfutil sdk-manager toolchain launch --ncs-version v3.1.0-preview1 --shell
cd ~/ncs
west init -m https://github.com/nrfconnect/sdk-nrf --mr v3.1.0-preview1 v3.1.0-preview1
cd v3.1.0-preview1
west update
west zephyr-export
```

```
west init -m https://github.com/ciniml/sdk-nrf --mr v3.1.0-preview1-rust v3.1.0-preview1-rust
west update
west zephyr-export
```

### Enable Rust support module

NnRF Connect SDKではホワイトリスト方式でZephyrのモジュールのうち有効なモジュールを管理している。
これらのモジュールは、NnRF Connect SDKのwestマニフェストの中に記載されている。

Rustの言語サポートモジュール `zephyr-lang-rust` は有効なモジュールのリストに入っていないため、ZephyrのRustサポートのドキュメント通りに実行しても **Rustサポートモジュールを有効化できない。**
対策として、インストールしたNnRF Connect SDKのwestマニフェストを修正してホワイトリストに `zephyr-lang-rust` を追加する。

`~/ncs/v3.1.0-preview1/nrf/west.yml` の `manifest/projects[zephyr]/import/name-allowlist` のリストに `zephyr-lang-rust` を追加すればよい。
現在のリビジョンだと、113行目として追加する。

```yaml
manifest:
...
  projects:
    - name: zephyr
      repo-path: sdk-zephyr
      revision: b1c76998edb2252cc10bb911ca7a9973972d1888
      import:
...
        name-allowlist:
          - percepio
...
          - zcbor
          - zephyr-lang-rust    // 113行目
          - zscilib
```

追加後、`zephyr-lang-rust` モジュールを有効化する。 [^1]

```
west config manifest.project-filter +zephyr-lang-rust
west update
```

[^1]: https://docs.zephyrproject.org/latest/develop/languages/rust/index.html#enabling-rust-support


## Build

### Open a terminal for build

```
nrfutil sdk-manager toolchain launch --ncs-version v3.1.0-preview1 --shell
source ~/ncs/v3.1.0-preview1/zephyr/zephyr-env.sh
```

### Build for Seeed XIAO nRF52840

```
west build -p -b xiao_ble/nrf52840/sense -- -DCONF_FILE=prj.conf -DDTC_OVERLAY_FILE=xiao_nrf52840.overlay
```

## rust-analyzer

```json
"rust-analyzer.linkedProjects": [
    "${userHome}/ncs/v3.1.0-preview1/modules/lang/rust/zephyr/Cargo.toml",
    "${userHome}/ncs/v3.1.0-preview1/modules/lang/rust/zephyr-build/Cargo.toml",
    "${userHome}/ncs/v3.1.0-preview1/modules/lang/rust/zephyr-sys/Cargo.toml"
],
"rust-analyzer.cargo.extraEnv": {
    "ZEPHYR_BASE": "${userHome}/ncs/v3.1.0-preview1/zephyr",
},
```

## References

* NCS installation - https://docs.nordicsemi.com/bundle/ncs-latest/page/nrf/installation.html
* Build command - https://docs.nordicsemi.com/bundle/ncs-latest/page/nrf/app_dev/config_and_build/cmake/index.html