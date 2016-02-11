#
# Makefile, primarily for convenience.
#
.PHONY: prepare clean compile eunit ct dev rel merge_records

NITRO_DIR = apps/tanuki_backend/priv/static/nitrogen

prepare:
	rebar get-deps
	cd deps/lager && $(MAKE)
	rebar -r prepare-deps

clean:
	rebar -r clean skip_deps=true

compile:
	@(test -d $(NITRO_DIR) || mkdir $(NITRO_DIR))
	$(MAKE) -C apps/tanuki_backend copy-static
	rebar -r compile skip_deps=true

eunit: compile
	rebar -r eunit skip_deps=true

ct: compile
	rebar -r ct skip_deps=true

dev: compile
	relx --dev-mode --relname tanuki --relvsn dev

rel: compile
	relx

merge_records:
	cd apps/merge_records && rebar compile escriptize
