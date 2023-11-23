#!/bin/bash

git clone https://github.com/rustdelhi/llm-writer.git && cd llm-writer

download_url="https://huggingface.co/rustformers/open-llama-ggml/resolve/main/open_llama_3b-f16.bin"

download_path="./assets/open_llama_3b-f16.bin"

mkdir -p ./assets

wget -O $download_path $download_url

echo "File downloaded successfully and saved to ${download_path}"
