FROM docker.cloudpss.net/cloudpss/buildpack AS builder

RUN npm install -g @vscode/vsce

ADD build/mirascript-vscode.tgz /mirascript-vscode
WORKDIR /mirascript-vscode/package

RUN --mount=type=secret,target=/root/.npmrc \
  npm install --omit=dev

RUN vsce package

FROM scratch

COPY --from=builder /mirascript-vscode/package/*.vsix /
