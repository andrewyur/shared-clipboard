{
    inputs = {
        nixpkgs.url = "nixpkgs/nixos-unstable";
        flake-utils.url = "github:numtide/flake-utils";
    };

    outputs = {self, flake-utils, nixpkgs}: 
        flake-utils.lib.eachDefaultSystem (system:
        let
            pkgs = import nixpkgs {
                inherit system;
                config = {
                    # android_sdk.accept_license = true;
                    allowUnfree = true;
                };
            };
            # androidSdk = pkgs.androidenv.androidPkgs_9_0.androidsdk;
        in {
            devShell = pkgs.mkShell {
                packages = with pkgs; [
                    # flutter
                    # androidSdk

                    # rust
                    cargo
                    clippy
                    rustc
                    rustfmt
                    rustup
                    lld_19
                ];
                shellHook =''
                export RUST_SRC_PATH=${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}
                zsh'';
            };
        }
    );
}
