SHELL := /bin/bash

IDL_FILE := artifacts/private-airdrop-idl.json
PROGRAM_BIN := target/riscv32im-risc0-zkvm-elf/docker/private_airdrop.bin
PROGRAM_BIN_ABS := $(abspath $(PROGRAM_BIN))
PROGRAM_SRC := methods/guest/src/bin/private_airdrop.rs

.PHONY: help build idl inspect deploy clean

help:
	@echo "LP-0003 Private Airdrop SPEL program"
	@echo "  make build    Build the RISC Zero guest binary"
	@echo "  make idl      Generate SPEL IDL"
	@echo "  make inspect  Inspect the built program binary"
	@echo "  make deploy   Deploy through logos-scaffold wallet wrapper"

build:
	cargo risczero build --manifest-path methods/guest/Cargo.toml

idl:
	spel generate-idl $(PROGRAM_SRC) > $(IDL_FILE)

inspect:
	@test -f "$(PROGRAM_BIN)" || (echo "ERROR: Binary not found. Run 'make build' first."; exit 1)
	cd /tmp && spel inspect "$(PROGRAM_BIN_ABS)"

deploy:
	@test -f "$(PROGRAM_BIN)" || (echo "ERROR: Binary not found. Run 'make build' first."; exit 1)
	logos-scaffold wallet -- deploy-program $(PROGRAM_BIN)

clean:
	rm -f $(IDL_FILE)
