# MyCondominium Backend

## Docs

- [**Rustdoc**](https://mycondominium-backend-b007cf.al3xdev.io/rustdoc/mycondominium_backend/index.html)
- [**OpenAPI/Swagger (Static)**](https://mycondominium-backend-b007cf.al3xdev.io/swagger/index.html)

## Environments

- [**Development Branch**](https://git.al3xdev.com/puc/software-engineering-v/mycondominium-backend/-/environments/2)
- [**Main Branch**](https://git.al3xdev.com/puc/software-engineering-v/mycondominium-backend/-/environments/3)

## Pre-Compiled Binaries

- [**GitLab Releases**](https://git.al3xdev.com/puc/software-engineering-v/mycondominium-backend/-/releases)

## Building

### Dependencies

<details>
<summary>Debian GNU/Linux</summary>

```shell
sudo apt install libssl-dev libpq-dev pkg-config build-essential
```
</details>

### Build

```shell
cargo build
```

### Run

```shell
cargo run -- daemon
```

## Usage

**Copy the template configuration:**
 
```shell
cp res/config_template.yaml config.yaml
```

**Or specify the configuration file path:**

```shell
cargo run -- daemon -c <file path>
```
