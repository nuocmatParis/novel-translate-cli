.PHONY: build run test vet install

APP_BINARY = bin/novel-translate-cli$(if $(filter Windows_NT,$(OS)),.exe,)

build:
	go build -trimpath -ldflags="-s -w" -o $(APP_BINARY) ./cmd/novel-translate-cli

run:
	go run -trimpath -ldflags="-s -w" ./cmd/novel-translate-cli tui

test:
	go test -buildvcs=false -count=1 ./...

vet:
	go vet ./...

install:
	sh scripts/install.sh
