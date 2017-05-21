#!/bin/bash
# Travis CI test runner


if [ "${TRAVIS_EVENT_TYPE}" = "cron" -a "${TARGET}" = "x86_64-unknown-linux-gnu" ] ; then
  make nightly-environment && make test
else
  cargo test --verbose
fi
