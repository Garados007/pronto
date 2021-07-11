# Build the rust web server with pronto

# this build is optimized for fast rust builds if the docker cache exists
FROM rust as builder
WORKDIR /usr/src/pronto
RUN mkdir src && \
    echo "fn main() { println!(\"empty build\"); }" > src/main.rs
COPY ./Cargo.lock ./
COPY ./Cargo.toml ./
RUN cargo build --release && \
    rm -rf src && \
    rm -rf target/release/.fingerprint/pronto-* && \
    rm -rf target/release/deps/pronto-* && \
    rm -rf target/release/pronto*
COPY ./migrations ./migrations
COPY ./src ./src
COPY ./diesel.toml ./
RUN cargo build --release

# Build swagger documentation

FROM debian as converter
WORKDIR /app
RUN apt-get update && \
    apt-get install -y jq python3-pip sed && \
    rm -rf /var/lib/apt/lists/* && \
    pip3 install yq
COPY ./open-api-v1.yml ./
COPY ./resources/api-index-template.html ./
RUN yq . ./open-api-v1.yml > ./open-api-v1.json && \
    line=$(grep -n 'var spec = {};' api-index-template.html | cut -d ":" -f 1) && \
    { \
        head -n $(($line-1)) api-index-template.html; \
        echo -n "        var spec = "; \
        cat open-api-v1.json; \
        tail -n +$(($line+1)) api-index-template.html; \
    } > index.html

# Create final runtime container

FROM debian
WORKDIR /app
RUN apt-get update && \
    apt-get install -y openssl libpq-dev && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/pronto/target/release/pronto /usr/local/bin/pronto
COPY --from=converter /app/index.html /app/doc.html
COPY ./open-api-v1.yml ./
COPY --from=converter /app/open-api-v1.json ./

EXPOSE 5000
CMD [ "pronto" ]
