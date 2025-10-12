# Sagittarius

## Development

### Requirements

- [devenv](https://devenv.sh/)
- [direnv](https://direnv.net/)(optional but recommended)

### Setup

With `direnv`:

```bash
direnv allow
```

Without `direnv`:

```bash
devenv shell
```

### Running

```bash
docker compose up
```

## Building

The docker images is built automatically with GitHub Actions on every push to `main` branch and on every release.
To build the image locally, run:

```bash
docker build -t sagittarius .
```

## Testing

### Backend

```bash
cargo nextest run --workspace --all-features
```
