FROM docker.cloudpss.net/cloudpss/buildpack-lts AS builder

RUN npm install -g @napi-rs/cli

COPY . /mirascript
WORKDIR /mirascript/packages/napi

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/mirascript/target \
    <<EOF
    set -e

    mkdir -p dist
    pnpm run build:debug --release
EOF

FROM scratch

COPY --from=builder /mirascript/packages/napi/dist /
