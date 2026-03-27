#!/bin/bash

## Update certificates.
update-ca-certificates

## Let the actual command take over
exec "$@"