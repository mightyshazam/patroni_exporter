# patroni_exporter

Basic Prometheus exporter for [Patroni](https://github.com/zalando/patroni).

Currently only supports Consul as a DCS but extending would be reasonably trivial. It currently serves my use case monitoring Patroni running on Hashicorp Nomad where Consul service registration is managed by Nomad. It's untested against Patroni-registered services but it would probably Just Work (tm).

## Installation

Binaries for Linux and macOS can be found under [Releases](https://github.com/ccakes/patroni_exporter/releases) - grab the latest for your platform there.

## Usage

patroni_exporter can be configured either by passing in arguments or by environment variables

```shell
$ patroni_exporter -h
patroni-exporter 0.3.0
Export Patroni metrics to Prometheus

USAGE:
    patroni_exporter [FLAGS] [OPTIONS] --consul <consul-url> --service <service>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v               Logging verbosity

OPTIONS:
    -t, --token <consul-token>    Consul token [env: CONSUL_HTTP_TOKEN=]
    -c, --consul <consul-url>     Consul URL [env: CONSUL_HTTP_ADDR=]
    -l, --listen <listen-addr>    HTTP listen address [default: 0.0.0.0:9393]
    -s, --service <service>       Patroni service name [env: PATRONI_SERVICE=]
```

## License

[MIT](/LICENSE)