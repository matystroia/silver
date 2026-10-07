FROM rust:latest
RUN apt-get update && apt-get install -y \
    libavcodec-dev libavformat-dev libavfilter-dev libavdevice-dev \
    libswscale-dev libswresample-dev libchafa-dev libglib2.0-dev pkg-config \
    clang
