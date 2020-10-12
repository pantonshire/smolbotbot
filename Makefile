GO := go
GO_BUILD := $(GO) build

build: migrator

clean:
	rm migrator

migrate: migrator
	@ ./$< -c config.json

migrator: config.json
	$(GO_BUILD) -o $@ cmd/migrate/migrate.go

config.json:
	cp default/default_config.json $@

.PHONY: build clean run