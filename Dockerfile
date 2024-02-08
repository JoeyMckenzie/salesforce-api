ARG RUST_VERSION=1.76.0
ARG DEBIAN_VERSION=11.6

FROM rust:${RUST_VERSION}-slim-buster as build

# args for the build stage
ARG VERSION="undefined"
ARG COMMIT_SHA="undefined"
ENV VERSION=$VERSION \
    COMMIT_SHA=$COMMIT_SHA \
    CONFIG_LEVEL=$CONFIG_LEVEL

# set working directory
WORKDIR /app

# copy source code
COPY . .

# on rebuilds, we explicitly cache our rust build dependencies to speed things up
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/rustup \
    set -eux; \
    export DEBIAN_FRONTEND=noninteractive; \
    apt-get update; \
    apt-get upgrade -y; \
    apt-get install -y pkg-config libssl-dev; \
    apt-get clean autoclean; \
    apt-get autoremove -y; \
    cargo build --release; \
    objcopy --compress-debug-sections target/release/salesforce_api ./salesforce_api

# run tests
# RUN cargo run test

# generate .version file
RUN echo "{\"version\":\"$VERSION\",\"commit\":\"$COMMIT_SHA\"}" > /app/.version

# entrypoint to allow for testing when this stage is build target
# ENTRYPOINT ["cargo", "run", "test"]


# stage two - we'll utilize a second container to run our built binary from our first container - slim containers!
FROM debian:${DEBIAN_VERSION}-slim as deploy

# export http port
EXPOSE 80

# set default working directory
WORKDIR /var/task

# optional build args
ARG ACM_PRIVATE_CERTIFICATE_AUTHORITY
ARG AWS_CONTAINER_CREDENTIALS_RELATIVE_URI

# install dependencies and aws cli
RUN set -eux; \
    export DEBIAN_FRONTEND=noninteractive; \
    apt-get update; \
    apt-get upgrade -y; \
    apt-get install -y --no-install-recommends openssl ca-certificates curl jq unzip; \
    curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"; \
    unzip -q awscliv2.zip; \
    ./aws/install; \
    apt autoremove -y; \
    rm -rf /var/lib/apt/lists/* ./aws*; \
    rm -rf /var/lib/{apt,dpkg,cache,log}/;

# use aws cli to fetch private ca cert and install it
RUN set -ex \
 && if [ ${ACM_PRIVATE_CERTIFICATE_AUTHORITY+x} ]; then \
    REGION=$(echo $ACM_PRIVATE_CERTIFICATE_AUTHORITY | cut -d':' -f4) \
 && aws --region $REGION acm-pca \
    get-certificate-authority-certificate \
    --certificate-authority-arn $ACM_PRIVATE_CERTIFICATE_AUTHORITY \
    2>&1 \
    | jq -r '.Certificate' \
    > /usr/local/share/ca-certificates/private-ca-devops.crt \
 && update-ca-certificates; fi

# copy build application
COPY --from=build /app/salesforce_api .

# set default command
CMD ["./salesforce_api"]