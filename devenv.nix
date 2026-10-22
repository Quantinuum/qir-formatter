{ pkgs, lib, config, inputs, ... }:

{
  # https://devenv.sh/basics/
  env.GREET = "devenv";
  env.SSL_CERT_FILE = "${pkgs.cacert.override {
    blacklist = [
      "emSign ECC Root CA - G3"
      "Comodo AAA Services root"
    ];
  }}/etc/ssl/certs/ca-bundle.crt";
  env.CARGO_HTTP_CAINFO = config.env.SSL_CERT_FILE;
  env.CURL_CA_BUNDLE = config.env.SSL_CERT_FILE;
  env.GIT_SSL_CAINFO = config.env.SSL_CERT_FILE;
  env.NIX_SSL_CERT_FILE = config.env.SSL_CERT_FILE;

  # https://devenv.sh/packages/
  packages = [ pkgs.git ];

  # https://devenv.sh/languages/
  languages.rust.enable = true;

  # https://devenv.sh/processes/
  # processes.dev.exec = "${lib.getExe pkgs.watchexec} -n -- ls -la";

  # https://devenv.sh/services/
  # services.postgres.enable = true;

  # https://devenv.sh/scripts/
  scripts.hello.exec = ''
    echo hello from $GREET
  '';

  # https://devenv.sh/basics/
  enterShell = ''
    hello         # Run scripts directly
    git --version # Use packages
  '';

  # https://devenv.sh/tasks/
  # tasks = {
  #   "myproj:setup".exec = "mytool build";
  #   "devenv:enterShell".after = [ "myproj:setup" ];
  # };

  # https://devenv.sh/tests/
  enterTest = ''
    echo "Running tests"
    git --version | grep --color=auto "${pkgs.git.version}"
  '';

  # https://devenv.sh/git-hooks/
  # git-hooks.hooks.shellcheck.enable = true;

  # See full reference at https://devenv.sh/reference/options/

  dotenv.enable = true;
}
