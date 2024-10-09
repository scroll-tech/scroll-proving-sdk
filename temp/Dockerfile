FROM scrolltech/sdk-cloud-prover:latest

COPY config.json /sdk_prover/
COPY temp/sdk_prover_run.sh /bin/

ENTRYPOINT ["sh", "/bin/sdk_prover_run.sh"]
