#!/bin/bash
while getopts c: flag
do
  case "${flag}" in
    c) component=${OPTARG};;
  esac
done

echo "Building and deploying component: rft-$component";

docker build . -f rft-$component/Dockerfile -t localhost:5000/rft-$component:latest
docker push localhost:5000/rft-$component:latest
sleep 1
kubectl rollout restart deployment rft-$component