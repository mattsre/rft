# r5t

## Development Priorities

- Worker SDK (Python) - Used for sourcing r5t data in jobs, i.e. `r5t.init()`
- Move r5t components to Helm chart, use Skaffold (or similar tools) for local development rather than running outside of the cluster
- Clean up error handling throughout the service
- Standardize on logging library/format, improve logging throughout project

## Architecture

![Current Architecture Diagram](docs/content/r5t-architecture-diagramV2.png)

## Developing Locally

Deploy the Helm chart:
```
cd r5t-chart/
helm install r5t .
```

Expose the Redis service:
```
export REDIS_PASSWORD=$(kubectl get secret --namespace default r5t-redis -o jsonpath="{.data.redis-password}" | base64 --decode)
kubectl port-forward --namespace default svc/r5t-redis-master 6379:6379
```

Run the gateway service locally:
```
cargo run --bin r5t-gateway
```

Run the controller service locally:
```
cargo run --bin r5t-controller
```

At this point, you have a working r5t deployment. To interact, you can build the CLI and use it.
```
cargo build --bin r5t-client
target/debug/r5t-client run -f test.py -p start_date=1980,1990,2000 end_date=2020,2025,2030 --format pairs
```