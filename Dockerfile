FROM --platform=$BUILDPLATFORM golang:1.25.5 AS builder

WORKDIR /src

ENV CGO_ENABLED=0 GOWORK=off

ARG TARGETOS
ARG TARGETARCH

COPY go.mod go.sum ./
RUN go mod download

COPY . .

RUN GOOS=$TARGETOS GOARCH=$TARGETARCH \
    go build -trimpath -ldflags="-s -w" \
    -o /out/novel-translate-cli \
    ./cmd/novel-translate-cli

FROM alpine:3.22

RUN apk add --no-cache \
    ca-certificates \
    tzdata \
    openjdk17-jre-headless \
    font-liberation

WORKDIR /workspace

COPY --from=builder /out/novel-translate-cli /usr/local/bin/novel-translate-cli

ENTRYPOINT ["novel-translate-cli"]
