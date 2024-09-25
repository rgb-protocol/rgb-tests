# syntax=docker.io/docker/dockerfile:1.7-labs
#
# required to support COPY --exclude
# see https://github.com/moby/buildkit/blob/dockerfile/1.7.0-labs/frontend/dockerfile/docs/reference.md#copy---exclude

FROM rust:1.91-slim-trixie

# requirements
RUN apt-get update && apt install -y --no-install-recommends \
    adduser ca-certificates curl libssl-dev pkg-config procps \
    && apt-get clean && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

# docker apt sources
RUN install -m 0755 -d /etc/apt/keyrings \
    && curl -fsSL https://download.docker.com/linux/debian/gpg \
    -o /etc/apt/keyrings/docker.asc \
    && chmod a+r /etc/apt/keyrings/docker.asc \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/debian \
    $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | \
    tee /etc/apt/sources.list.d/docker.list > /dev/null

# docker installation
RUN apt-get update && apt install -y --no-install-recommends \
        docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin \
        gosu tini \
    && apt-get clean && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

# create unprivileged user
ENV USR="user" WORKDIR="/opt"
RUN adduser --shell /bin/bash --disabled-login --gecos "user" $USR \
    && adduser $USR docker \
    && chown -R $USR:$USR $WORKDIR

WORKDIR $WORKDIR
USER $USR

# copy submodules and test files
COPY --exclude=entrypoint.sh . .

USER root
COPY entrypoint.sh .
ENTRYPOINT [ "tini", "--", "/opt/entrypoint.sh" ]
