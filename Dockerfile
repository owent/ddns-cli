FROM docker.io/alpine:latest

LABEL maintainer "OWenT <admin@owent.net>"

RUN mkdir -p "/usr/local/ddns-cli/bin"
COPY "./bin/ddns-cli" "/usr/local/ddns-cli/bin"
ENV PATH="/usr/local/ddns-cli/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"

CMD ["/usr/local/ddns-cli/bin/ddns-cli"]
