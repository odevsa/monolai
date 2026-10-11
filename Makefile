# Monolai - Makefile wrapper forwarding to just (https://github.com/casey/just)
# All canonical tasks are defined in justfile.

.PHONY: default
default:
	@just

ifeq (version,$(firstword $(MAKECMDGOALS)))
  VERSION_ARG := $(wordlist 2,$(words $(MAKECMDGOALS)),$(MAKECMDGOALS))
  $(eval $(VERSION_ARG):;@:)
endif

version:
	@just version $(VERSION_ARG)

%:
	@just "$@"
