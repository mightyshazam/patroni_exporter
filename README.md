# patroni_exporter

Basic Prometheus exporter for [Patroni](https://github.com/zalando/patroni).

Currently only supports Consul as a DCS but extending would be reasonably trivial. It currently serves my use case monitoring Patroni running on Hashicorp Nomad where Consul service registration is managed by Nomad. It's untested against Patroni-registered services but it would probably Just Work (tm).

## Usage

patroni_exporter can be configured either by passing in arguments or by environment variables

```shell
$ patroni_exporter -h
patroni-exporter 0.2.0
Export Patroni metrics to Prometheus

USAGE:
    patroni_exporter [FLAGS] [OPTIONS] --consul <consul-url> --listen <listen-addr> --service <service>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v               Logging verbosity

OPTIONS:
    -t, --token <consul-token>    Consul token [env: CONSUL_HTTP_TOKEN=]
    -c, --consul <consul-url>     Consul URL [env: CONSUL_HTTP_ADDR=]
    -l, --listen <listen-addr>    HTTP listen address
    -s, --service <service>       Patroni service name [env: PATRONI_SERVICE=]
```

## License

[MIT](/LICENSE)