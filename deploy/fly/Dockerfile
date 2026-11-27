# The persistent multiplayer server (`openspec/changes/multiplayer` design
# 27), the Tenebris way (`tenebris-rs/Dockerfile`): stage 1 builds the
# dedicated server, stage 2 runs only the binary.
#
# The build context is the repository root: the workspace's manifests and
# the assets the core reads at compile time (`include_str!` of
# assets/config, climate, locale and shaders). The root .dockerignore keeps
# target/, docs/ and the rest out of it.
#
#   docker build -f deploy/fly/Dockerfile -t pbd-server .
#   fly deploy --config deploy/fly/fly.toml --dockerfile deploy/fly/Dockerfile --ha=false
FROM rust:bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY assets/config ./assets/config
COPY assets/climate ./assets/climate
COPY assets/locale ./assets/locale
COPY assets/shaders ./assets/shaders
RUN cargo build --release --locked -p pbd-headless

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/pbd-headless /usr/local/bin/pbd-headless
# The world lives on the volume at /data (fly.toml), so it survives every
# deploy. A new volume starts a new world with this seed; an existing world
# keeps its own. Players start empty-handed, as a new single-player world
# does (--kit 0).
EXPOSE 47123/udp
CMD ["pbd-headless", "serve", "--bind", "fly-global-services", "--port", "47123", "--world", "/data/worlds/main", "--seed", "1", "--kit", "0"]
