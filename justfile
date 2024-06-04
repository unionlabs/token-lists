set dotenv-load
set positional-arguments

per:
  sudo chown -R $USER .

default: per

format:
  biome format . \
    --log-level="info" \
    --log-kind="pretty" \
    --error-on-warnings \
    --diagnostic-level="info" \
    --write

lint:
  biome lint . \
    --log-level="info" \
    --log-kind="pretty" \
    --error-on-warnings \
    --diagnostic-level="info" \
    --apply-unsafe

format-nix:
	nixfmt *.nix **/*.nix **/**/*.nix --width=100

fmt: format format-nix
fml: fmt lint
