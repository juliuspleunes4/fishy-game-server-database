{
  description = "Team dino flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };

    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, crane, fenix, flake-utils, advisory-db, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        inherit (pkgs) lib;

        craneLib = crane.mkLib pkgs;
        src = ./backend;

        commonArgs = {
          inherit src;
          strictDeps = true;

          buildInputs = [
            # Add additional build inputs here
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];

          SQLX_OFFLINE = "1";
        };

        craneLibLLvmTools = craneLib.overrideToolchain
          (fenix.packages.${system}.complete.withComponents [
            "cargo"
            "llvm-tools"
            "rustc"
            "rust-src"
          ]);

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        backend = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });

        # Database Configuration
        dbName = "fishydb";
        dbUser = "fishyuser";
        dataDir = "./.pg_data";
        passFile = "./.pg_password";

        # Database Script (App)
        dbScript = pkgs.writeShellScriptBin "fishy-db" ''
           export PGDATA="${dataDir}"
           export PGHOST="$PWD/postgres_socket"
          
           mkdir -p "$PGHOST"

           if [ ! -d "$PGDATA" ]; then
             echo "Initializing postgres data directory..."
             ${pkgs.postgresql_16}/bin/initdb --auth=trust --no-locale --encoding=UTF8 > /dev/null

             ${pkgs.openssl}/bin/openssl rand -base64 12 > "${passFile}"
             PASS=$(cat "${passFile}")

             echo "Setting up User and DB..."
             ${pkgs.postgresql_16}/bin/pg_ctl start -w -o "-k '$PGHOST' -p 5432"
            
             ${pkgs.postgresql_16}/bin/createuser -h "$PGHOST" --superuser ${dbUser}
             ${pkgs.postgresql_16}/bin/createdb -h "$PGHOST" -O ${dbUser} ${dbName}
             ${pkgs.postgresql_16}/bin/psql -h "$PGHOST" -c "ALTER USER ${dbUser} WITH PASSWORD '$PASS';" postgres
            ${pkgs.postgresql_16}/bin/psql -h "$PGHOST" -d ${dbName} -f ./backend/database-init.sql
             ${pkgs.postgresql_16}/bin/pg_ctl stop
           fi

           PASS=$(cat "${passFile}")
           DB_URL="postgres://${dbUser}:$PASS@127.0.0.1:5432/${dbName}"
         
          # Update backend/.env (Remove old URL if exists, then append new one)
           touch ./backend/.env
           ${pkgs.gnused}/bin/sed -i '/^DATABASE_URL=/d' ./backend/.env
           echo "DATABASE_URL=$DB_URL" >> ./backend/.env
         
           echo "Socket: $PGHOST"
          
           exec ${pkgs.postgresql_16}/bin/postgres -k "$PGHOST" -p 5432
        '';

      in
      {
        checks = {
          inherit backend;

          my-crate-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          my-crate-fmt = craneLib.cargoFmt {
            inherit src;
          };

          my-crate-toml-fmt = craneLib.taploFmt {
            src = pkgs.lib.sources.sourceFilesBySuffices src [ ".toml" ];
          };

          my-crate-audit = craneLib.cargoAudit {
            inherit src advisory-db;
          };

          my-crate-deny = craneLib.cargoDeny {
            inherit src;
          };
        };

        packages = {
          default = backend;
        } // lib.optionalAttrs (!pkgs.stdenv.isDarwin) {
          my-crate-llvm-coverage = craneLibLLvmTools.cargoLlvmCov (commonArgs // {
            inherit cargoArtifacts;
          });
        };

        apps = {
          default = flake-utils.lib.mkApp {
            drv = backend;
          };
          # Run: nix run .#db
          db = flake-utils.lib.mkApp {
            drv = dbScript;
            name = "fishy-db";
          };
        };

        devShells.default = craneLibLLvmTools.devShell {
          checks = self.checks.${system};

          shellHook = ''
            if [ -f ./backend/.env ]; then
              export $(cat ./backend/.env | xargs)
            fi

            echo "🦀 Welcome to fishy backend 🦀"
            echo "Run 'nix run .#db' to start the database."
            echo "Run 'cargo run ./backend' to start the backend."
          '';

          packages = [
            pkgs.sqlx-cli
            pkgs.postgresql_16
            pkgs.openssl
          ];
        };
      });
}
