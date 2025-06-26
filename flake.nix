{
  description = "TivanderIT: Leptos SSR Website";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      flake-utils,
      ...
    }:
    let
      # === Production Module Definition ===
      # Thid module is intended to be imported into a NixOS configuration.
      nixosModule =
        {
          config,
          lib,
          pkgs,
          ...
        }:
        with lib;
        let
          cfg = config.services.tivanderit-web;
          tivanderitPackage = self.packages.${pkgs.system}.default;
        in
        {
          # Service options for configuration.nix
          options.services.tivanderit-web = {
            enable = mkEnableOption "TivanderIT Web server";
            port = mkOption {
              type = types.port;
              default = 3000;
              description = "Port for the web service to listen on.";
            };
            address = mkOption {
              type = types.str;
              default = "127.0.0.1";
              description = "Address for the web service to listen on.";
            };
            databasePath = mkOption {
              type = types.path;
              default = "/var/lib/tivanderit-web/tivanderit.db";
              description = "Path to the SQLite database file.";
            };
          };

          # Service configuration
          config = mkIf cfg.enable {
            # Create a dedicated user/group
            users.users.tivanderit-web = {
              isSystemUser = true;
              group = "tivanderit-web";
              home = "/var/lib/tivanderit-web"; # Corresponds to StateDirectory
            };
            users.groups.tivanderit-web = { };

            # Systemd service definition
            systemd.services.tivanderit-web = {
              description = "TivanderIT Web server service";
              after = [ "network.target" ];
              wantedBy = [ "multi-user.target" ];

              serviceConfig = {
                ExecStart = "${tivanderitPackage}/bin/tivanderit";
                Restart = "always";
                User = "tivanderit-web";
                Group = "tivanderit-web";
                StateDirectory = "tivanderit-web";
                StateDirectoryMode = "0750";
                WorkingDirectory = "/var/lib/tivanderit-web";
                # Security hardening options
                ProtectSystem = "strict";
                ProtectHome = "true";
                PrivateTmp = true;
                NoNewPrivileges = true;
              };

              # Environment variables needed by the running service
              environment = {
                LEPTOS_OUTPUT_NAME = tivanderitPackage.pname;
                LEPTOS_SITE_ROOT = "${tivanderitPackage}/share/site";
                LEPTOS_SITE_ADDR = "${cfg.address}:${toString cfg.port}";
                DATABASE_URL = "sqlite:${cfg.databasePath}";
                MIGRATIONS_PATH = "${tivanderitPackage}/share/migrations";
              };
            };
          };
        };
    in
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        # --- System-specific packages and overlays ---
        overlays = [ inputs.rust-overlay.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config = {
            allowUnfree = true;
          };
        };

        # --- Read project metadata from Cargo.toml ---
        cargoTomlContent = builtins.readFile ./Cargo.toml;
        cargoTomlParsed = builtins.fromTOML cargoTomlContent;
        projectName = cargoTomlParsed.package.name or "tivanderit-web";
        projectVersion = cargoTomlParsed.package.version or "0.0.1";

        # --- Toolchain and Dependencies ---
        rustVersion = pkgs.rust-bin.stable."1.87.0".default.override {
          extensions = [ "rust-src" ];
          targets = [ "wasm32-unknown-unknown" ];
        };

        # Runtime dependencies (linked libraries)
        runtimeDeps = with pkgs; [
          openssl
          sqlite
        ];

        # Build dependencies (compilers, tools)
        buildDeps = with pkgs; [
          pkg-config
          trunk
          dart-sass
          cargo-leptos
          wasm-pack
          binaryen
        ];

        # Combined dependencies for nativeBuildInputs
        nativeBuildInputs = buildDeps ++ runtimeDeps;

        run-vm-headless = pkgs.writeShellScriptBin "run-vm-headless" ''
          #!${pkgs.runtimeShell}
          echo "Starting VM in headless mode..."
          exec ${self.nixosConfigurations.vm-test.config.system.build.vm}/bin/run-nixos-vm -nographic "$@"
        '';
      in
      {
        # === Packaging (Both production and test-VM) ===
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = projectName;
          version = projectVersion;

          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          rustc = rustVersion;
          cargo = rustVersion;

          buildInputs = runtimeDeps;
          nativeBuildInputs = nativeBuildInputs;

          # Environment variables needed by `cargo leptos build`
          LEPTOS_OUTPUT_NAME = projectName;
          LEPTOS_SITE_ROOT = "target/site";

          buildPhase = ''
            runHook preBuild
            export PATH="${pkgs.lib.makeBinPath buildDeps}:${rustVersion}/bin:$PATH"
            echo "Running cargo leptos build --release..."
            cargo leptos build --release
            runHook postBuild
          '';

          installPhase = ''
            runHook preInstall

            # Install server binary
            install -Dm755 "target/release/${projectName}" "$out/bin/${projectName}"

            # Install frontend assets
            local site_src_dir="target/site"
            local site_dest_dir="$out/share/site"
            if [ -d "$site_src_dir" ]; then
                mkdir -p "$site_dest_dir"
                cp -r "$site_src_dir"/* "$site_dest_dir/"
            else
                echo "Error: Site assets directory not found at '$site_src_dir' after build!" >&2
                exit 1
            fi

            # Install migrations
            local migrations_src_dir="./migrations"
            local migrations_dest_dir="$out/share/migrations"
            if [ -d "$migrations_src_dir" ]; then
              mkdir -p "$migrations_dest_dir"
              cp -r "$migrations_src_dir"/* "$migrations_dest_dir/"
            else
              echo "Warning: Migrations directory not found at '$migrations_src_dir'." >&2
              mkdir -p "$migrations_dest_dir"
            fi

            runHook postInstall
          '';

          meta = {
            description = "${projectName} - Leptos SSR Web Application";
            homepage = "https://tivanderit.se";
            license = pkgs.lib.licenses.unfree;
            platforms = pkgs.lib.platforms.linux ++ pkgs.lib.platforms.darwin;
          };
        };
        # === Development Environment ===
        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.packages.${system}.default ];

          packages = with pkgs; [
            rustVersion
            cargo-watch
            cargo-generate
            sqlx-cli
            leptosfmt
            sqlite-interactive
            git
            nodejs
            playwright-test
            playwright-driver
            typescript
            jq
            coreutils-full
            gnused
            gawk
          ];

          shellHook = ''
            echo "Entering Leptos development shell for ${projectName}..."
            export LEPTOS_OUTPUT_NAME="${projectName}"
            export LEPTOS_SITE_ROOT="target/site"
            export LEPTOS_SITE_ADDR="127.0.0.1:3000"
            export LEPTOS_RELOAD_PORT="3001"
            export DATABASE_URL="sqlite:tivanderit-dev.db"
            export MIGRATIONS_PATH="./migrations"
            export PLAYWRIGHT_BROWSERS_PATH=${pkgs.playwright-driver.browsers}
            export PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS=true

            echo "-----------------------------------------------------"
            echo " Development Environment Variables:"
            echo "   LEPTOS_OUTPUT_NAME=$LEPTOS_OUTPUT_NAME"
            echo "   LEPTOS_SITE_ROOT=$LEPTOS_SITE_ROOT"
            echo "   LEPTOS_SITE_ADDR=$LEPTOS_SITE_ADDR"
            echo "   LEPTOS_RELOAD_PORT=$LEPTOS_RELOAD_PORT"
            echo "   DATABASE_URL=$DATABASE_URL"
            echo "   MIGRATIONS_PATH=$MIGRATIONS_PATH"
            echo "-----------------------------------------------------"
            echo " Tool Versions:"
            echo "   Rust: $(rustc --version)"
            echo "   Cargo: $(cargo --version)"
            echo "   Cargo-Leptos: $(cargo-leptos --version || echo 'Not found')"
            echo "   Leptosfmt: $(leptosfmt --version || echo 'Not found or cargo-leptosfmt package needed')"
            echo "   Trunk: $(trunk --version || echo 'Not found')"
            echo "   wasm-pack: $(wasm-pack --version || echo 'Not found')"
            echo "   Cargo-Generate: $(cargo-generate --version || echo 'Not found')"
            echo "   SQLx-CLI: $(sqlx --version || echo 'Not found')"
            echo "-----------------------------------------------------"
            echo " Ready! Common commands:"
            echo "   cargo leptos watch          # Start dev server with auto-reload"
            echo "   cargo leptos build          # Build for development"
            echo "   leptosfmt .                 # Format Leptos code"
            echo "   ./update-versions.sh             # Run to sync versions in Cargo.toml and package.json"
            echo "-----------------------------------------------------"
          '';
        };

        # === Run Commands ===
        apps = {
          # App for running the compiled binary (nix run .)
          default = flake-utils.lib.mkApp {
            drv = self.packages.${system}.default;
          };

          # App for running test-VM (nix run .#vm-test)
          vm-test = flake-utils.lib.mkApp {
            drv = run-vm-headless;
          };
        };

      }
    )
    // {
      nixosModules.default = nixosModule;

      # === Test-VM ===
      nixosConfigurations.vm-test = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        specialArgs = { inherit inputs self; };

        modules = [
          self.nixosModules.default
          (
            { config, ... }:
            {
              imports = [
                "${nixpkgs}/nixos/modules/virtualisation/qemu-vm.nix"
              ];

              nixpkgs.config.allowUnfree = true;

              # Activate Tivander IT Web service.
              services.tivanderit-web.enable = true;

              # Activate Caddy service
              services.caddy = {
                enable = true;
                virtualHosts."http://tivanderit.test http://localhost" = {
                  extraConfig = ''
                    reverse_proxy localhost:${toString config.services.tivanderit-web.port}
                  '';
                };
              };

              services.qemuGuest.enable = true;

              users.users.testuser = {
                isNormalUser = true;
                initialPassword = "test";
                extraGroups = [ "wheel" ];
              };

              networking.firewall.allowedTCPPorts = [ 80 ];
              virtualisation.forwardPorts = [
                {
                  from = "host";
                  host.port = 8080;
                  guest.port = 80;
                }
              ];

              system.stateVersion = "24.05";
            }
          )
        ];
      };
    };
}
