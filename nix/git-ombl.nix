{
  rustPlatform,
  versionCheckHook,
  openssl,
  pkg-config,
}:
rustPlatform.buildRustPackage {
  pname = "git-ombl";
  version = "0.1.0";
  src = ../.;

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  nativeBuildInputs = [
    pkg-config
  ];
  buildInputs = [
    openssl
  ];

  # checkFlags = [
  # Color error for those tests as we are not in a tty
  # "--skip=formatters::colored::tests::test_colored_formatter_empty_histor"
  # "--skip=formatters::colored::tests::test_colored_formatter_with_entries"
  # ];

  nativeInstallCheckInputs = [ versionCheckHook ];
  versionCheckProgramArg = "--version";
  doInstallCheck = true;
}
