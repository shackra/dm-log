{ pkgs, ... }:

{
  # https://devenv.sh/languages/
  languages.rust = {
    enable = true;
    channel = "stable";
    lsp.enable = true;
  };

  packages = with pkgs; [
    fontconfig
    pkg-config
    sdl3
    sdl3-ttf
  ];

  enterTest = ''
    echo "Running tests"
    echo "Emacs (dm-log)"
    emacs -batch -L "$DEVENV_ROOT/lisp" -l dm-log-test.el -f ert-run-tests-batch-and-exit
    echo "Rust (mazaforja)"
    cargo test -p mazaforja --manifest-path rust/Cargo.toml
  '';

  git-hooks.hooks = {
    rustfmt = {
      enable = true;
      name = "rustfmt (rust/)";
      entry = ''cargo fmt --manifest-path ./rust/Cargo.toml --all'';
      files = "^rust/";
      language = "system";
      pass_filenames = false;
    };
  };
}
