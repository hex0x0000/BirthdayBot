#!/usr/bin/env bash

IMAGE="multiarch/debian-debootstrap:arm64-bullseye"

sudo docker run --privileged --rm multiarch/qemu-user-static:register

docker pull $IMAGE

docker run -ti --rm \
	--name CROSS \
        --network host \
        $IMAGE \
        "/bin/bash"
