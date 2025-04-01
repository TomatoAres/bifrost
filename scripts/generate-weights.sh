#!/usr/bin/env bash

## EXAMPLE
##
## Generate the weightInfo files of `bifrost-runtimes`;
# sh ./scripts/generate-weights.sh bifrost

# 1. Build all-release which is added with "runtime-benchmarks" feature;
make build-all-release-with-bench
# 2. Filter the pallets of ${runtime} that should be executed benchmark;
IFS=', ' read -r -a runtimes <<< $@;
for runtime in "${runtimes[@]}"
do
    # Special handling for bifrost-paseo, append '-local' suffix for other cases.
    if [ "$runtime" = "bifrost-paseo" ]; then
        chain="$runtime"
    else
        chain="${runtime}-local"
    fi
    echo $chain
    target/release/bifrost benchmark pallet --chain=$chain --list | sed -n '2,$p' | grep -Eio "^\w+" | uniq |
        while IFS= read -r line
        do
		  	pallet=$line
		  	temp=${pallet/bifrost_/}
		  	pallet_dir=${temp//_/-}
            pallet_path="./pallets/${pallet_dir}"
            if [ ! -d "$pallet_path" ]; then
                echo "Skipping non-existent pallet directory: $pallet_path"
                continue
            fi           
			echo "benchmark pallet ${pallet}"
			echo "benchmark runtime ${runtime}"
            mkdir -p "${pallet_path}/src"
			target/release/bifrost benchmark pallet --chain=$chain \
			--steps=50 \
			--repeat=20 \
			--pallet=$pallet \
			--extrinsic="*" \
			--execution=wasm \
			--wasm-execution=compiled \
			--heap-pages=4096 \
            --output="${pallet_path}/src/weights.rs" \
			--template="./weight-template/pallet-weight-template.hbs";
        done
done

IFS=', ' read -r -a runtimes <<< $@;
for runtime in "${runtimes[@]}"
do
    # Special handling for bifrost-paseo, append '-local' suffix for other cases.
    if [ "$runtime" = "bifrost-paseo" ]; then
        chain="$runtime"
    else
        chain="${runtime}-local"
    fi   
    runtime_weights_path="./runtime/${runtime}/src/weights"
    mkdir -p "$runtime_weights_path"  
    target/release/bifrost benchmark pallet --chain=$chain --list | sed -n '2,$p' | grep -Eio "^\w+" | uniq |
        while IFS= read -r line
        do
		  	pallet=$line
		  	temp=${pallet/bifrost_/}
		  	pallet_dir=${temp//_/-}
			echo "benchmark pallet ${pallet}"
			echo "benchmark runtime ${runtime}"
			target/release/bifrost benchmark pallet --chain=$chain \
			--steps=50 \
			--repeat=20 \
			--pallet=$pallet \
			--extrinsic="*" \
			--execution=wasm \
			--wasm-execution=compiled \
			--heap-pages=4096 \
			--output="$runtime_weights_path/${pallet}.rs" \
			--template="./weight-template/runtime-weight-template.hbs";
        done
done
