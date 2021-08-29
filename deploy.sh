#!/bin/bash
while getopts c: flag
do
  case "${flag}" in
    c) component=${OPTARG};;
  esac
done

echo "Building and deploying component: r5t-$component";

docker build . -f r5t-$component/Dockerfile -t localhost:5000/r5t-$component:latest
docker push localhost:5000/r5t-$component:latest
sleep 1
kubectl rollout restart deployment r5t-$component