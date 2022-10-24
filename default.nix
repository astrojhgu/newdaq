# default.nix
with import <nixpkgs> {};
stdenv.mkDerivation {
    name = "controller"; # Probably put a more meaningful name here
    buildInputs = [libpcap pkg-config fontconfig];
    LD_LIBRARY_PATH=libpcap+"/lib";
}
