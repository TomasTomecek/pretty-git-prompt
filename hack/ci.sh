#!/bin/bash
# Travis CI test runner

sudo chmod -R 0777 .
sudo chown -R 1000:1000 .
if [ "${TRAVIS_EVENT_TYPE}" = "cron" -a "${TARGET}" = "x86_64-unknown-linux-gnu" ] ; then
  make nightly-environment && make test
else
  cargo test --verbose
fi
