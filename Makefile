#
# Makefile, primarily for convenience.
#
.PHONY: deps clean compile test dev release

NITRO_DIR = apps/tanuki_backend/priv/static/nitrogen
VERSION = _rel/tanuki/Version

deps:
	rebar get-deps
	cd deps/lager && $(MAKE)
	rebar -r prepare-deps

clean:
	rebar -r clean skip_deps=true

compile:
	@(test -d deps || $(MAKE) deps)
	@(test -d $(NITRO_DIR) || mkdir $(NITRO_DIR))
	$(MAKE) -C apps/tanuki_backend copy-static
	rebar -r compile skip_deps=true

test: compile
	rebar -r ct skip_deps=true

dev: compile
	relx --dev-mode --relname tanuki --relvsn dev

release: clean compile
	relx
	@echo 'Build Date:' `date -R` > $(VERSION)
	@echo 'HEAD Commit:' `git log --max-count=1 --pretty='%h'` >> $(VERSION)
