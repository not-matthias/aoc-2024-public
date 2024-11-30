{
  pkgs,
  lib,
  config,
  inputs,
  ...
}: {
  # https://devenv.sh/languages/
  languages.rust = {
    enable = true;
    channel = "nightly";
    mold.enable = true;
  };

  # See full reference at https://devenv.sh/reference/options/
}
