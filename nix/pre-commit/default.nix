{ config }:
{
  check.enable = true;
  settings.hooks = {
    nil.enable = true;
    treefmt = {
      enable = true;
      package = config.treefmt.build.wrapper;
    };
  };
}
