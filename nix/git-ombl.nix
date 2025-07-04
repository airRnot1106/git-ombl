{
  lib,
  stdenv,
  rustPlatform,
  versionCheckHook,
  libiconv,
  openssl,
  pkg-config,
  darwin,
}:
rustPlatform.buildRustPackage {
  pname = "git-ombl";
  version = "0.1.1";
  src = ../.;

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  nativeBuildInputs = [
    pkg-config
  ];
  buildInputs =
    [
      libiconv
      openssl
    ]
    ++ lib.optionals stdenv.isDarwin [
      darwin.apple_sdk.frameworks.Security
    ];

  checkFlags = [
    # Skip integration tests as they require a git repository
    "--skip=test_sample_file_line_history_integration"
    "--skip=test_sample_file_complete_history_traversal"
    "--skip=test_sample_file_different_lines"
    "--skip=test_sample_file_with_all_formatters"
    "--skip=test_sample_file_commit_messages_and_authors"
    "--skip=test_sample_file_change_types"
    "--skip=test_sample_file_sort_order_integration"
    "--skip=test_sample_file_ignore_revisions_integration"
    "--skip=test_sample_file_date_filtering_integration"
    "--skip=test_sample_file_date_filtering_with_other_options"
    "--skip=test_sample_file_date_format_compatibility"
  ];

  nativeInstallCheckInputs = [ versionCheckHook ];
  versionCheckProgramArg = "--version";
  doInstallCheck = true;
}
