#!/usr/bin/env bash

IMAGE="multiarch/debian-debootstrap:arm64-bullseye"

sudo docker run --privileged --rm tonistiigi/binfmt --install qemu-aarch64

docker pull $IMAGE

docker run -ti --rm \
	--name CROSS \
        --network host \
        $IMAGE \
        "/bin/bash"
