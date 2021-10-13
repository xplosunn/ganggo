# ganggo

Cross-platform alternative for [dmenu](https://tools.suckless.org/dmenu/).

## Building etc

`nix build` will build the default package defined in the flake nix. A symlink to the nix store called `result` will be here. If you're in the dev shell which you can enter with `nix develop` (or with direnv installed `direnv allow`) has cargo and an lsp server installed. You can do the usual `cargo run` etc as well there.


