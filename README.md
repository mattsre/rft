# r5t

## Development Priorities

- Worker SDK (Python) - Used for sourcing r5t data in jobs, i.e. `r5t.init()`
- Move r5t components to Helm chart, use Skaffold (or similar tools) for local development rather than running outside of the cluster
- Clean up error handling throughout the service
- Standardize on logging library/format, improve logging throughout project

## Architecture

![Current Architecture Diagram](docs/content/r5t-architecture-diagramV2.png)

## Developing Locally

Create a local Kind cluster and Docker registry. There's a script for automating this:
```
./create-kind-cluster.sh
```

When making changes to the controller or gateway, you'll need to build/push images
to the local registry. This workflow is tedious right now, and should be improved:
```
# Must be ran from the root of the repository
docker build . -f r5t-controller/Dockerfile -t localhost:5000/r5t-controller:latest
docker build . -f r5t-gateway/Dockerfile -t localhost:5000/r5t-gateway:latest

docker push localhost:5000/r5t-controller:latest
docker push localhost:5000/r5t-gateway:latest
```

Deploy/Upgrade the Helm chart:
```
cd r5t-chart/

# For first time installation:
helm install r5t .

# For upgrading:
export REDIS_PASSWORD=$(kubectl get secret --namespace "default" r5t-redis -o jsonpath="{.data.redis-password}" | base64 --decode)
helm upgrade r5t --set redis.auth.password=$REDIS_PASSWORD .
```

Expose the gateway service:
```
kubectl port-forward --namespace default svc/r5t-gateway-svc 8000:8000
```

At this point, you have a working r5t deployment. To interact, you can build the CLI and use it.
```
cargo build --bin r5t-client
target/debug/r5t-client run -f test.py -p start_date=1980,1990,2000 end_date=2020,2025,2030 --format pairs
```