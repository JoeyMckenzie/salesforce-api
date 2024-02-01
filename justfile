set shell := ["pwsh.exe", "-c"]

default: clippy

# install dependencies for React and Laravel
clippy:
    cargo clippy

# install dependencies for React and Laravel
cw:
    cargo watch -x clippy

# install dependencies for React and Laravel
cr:
    cargo watch -x run