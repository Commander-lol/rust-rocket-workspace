FROM rustlang/rust:nightly AS build

RUN mkdir /project
WORKDIR /project

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

RUN USER=root cargo new --bin web

COPY ./web/Cargo.toml ./web/Cargo.toml

RUN cargo build --bin web --release

RUN rm ./target/release/deps/web* ./target/release/web*

COPY ./web ./web

RUN cargo build --bin web --release

# Web Target Image

FROM debian:jessie-slim AS web

WORKDIR /runtime

COPY --from=build /project/target/release/web .

# Also Copy Web Resources
COPY --from=build /project/web/public public/
COPY --from=build /project/web/templates templates/

ENV APP_PORT=80
ENV APP_ADDRESS=0.0.0.0
ENV APP_STATIC_DIR=/runtime/public

EXPOSE 80

CMD ./web
