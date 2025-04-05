.PHONY: start-local stop-local delete-local

build:
	turbo run build

dev:
	turbo run run

start-local-infra:
	turbo run start:infra --filter="@porti/local-*"

stop-local-infra:
	turbo run stop:infra --filter="@porti/local-*"

delete-local-infra:
	turbo run delete:infra --filter="@porti/local-*"

build-docker:
	turbo run build:infra --filter="@porti/local-*"
