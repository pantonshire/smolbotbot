GO := go
GO_BUILD := $(GO) build

ifndef $(BUILD_TAGS)
BUILD_TAGS := mysql
endif

build: migrator

clean:
	rm migrator

migrate: migrator config.json
	@ ./$< -c config.json

migrator:
	$(GO_BUILD) -tags $(BUILD_TAGS) -o $@ cmd/migrate/migrate.go

config.json:
	cp default/default_config.json $@

.PHONY: build clean run