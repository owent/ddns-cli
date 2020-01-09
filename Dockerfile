FROM docker.io/alpine:latest

LABEL maintainer "OWenT <admin@owent.net>"

COPY "./bin/ddns-cli" "/usr/local/bin/"

CMD ["/usr/local/bin/ddns-cli"]
