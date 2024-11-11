FROM gcr.io/distroless/cc-debian12

WORKDIR /app

COPY --chmod=0755 target/release/radar-server-standalone radar-server-standalone
COPY radar/web/dist www

ENTRYPOINT [ "/app/radar-server-standalone" ]
CMD [ "--static-dir", "/app/www/" ]