# Contributing

## Prerequisites

1. `substreams` binary

    ```bash
    # Use correct binary for your platform
    LINK=$(curl -s https://api.github.com/repos/streamingfast/substreams/releases/latest | awk '/download.url.*linux/ {print $2}' | sed 's/"//g')
    curl -L  $LINK  | tar zxf -
    # mkdir ~/.local/bin 
    mv substreams ~/.local/bin/substreams
    substreams --version # 1.0.1
    ```

2. API key from [https://app.streamingfast.io/](https://app.streamingfast.io/)

    ```bash
    sudo apt-get install jq
    export STREAMINGFAST_KEY=server_******* # Use your own API key
    export SUBSTREAMS_API_TOKEN=$(curl https://auth.streamingfast.io/v1/auth/issue -s --data-binary '{"api_key":"'$STREAMINGFAST_KEY'"}' | jq -r .token)
    ```

    ```bash
    substreams run -e mainnet.eth.streamingfast.io:443 https://github.com/streamingfast/substreams-ethereum-quickstart/releases/download/1.0.0/substreams-ethereum-quickstart-v1.0.0.spkg map_block --start-block 12292922 --stop-block +1
    ```

3. Environment deps

    ```bash
    npm install -g @bufbuild/buf
    buff --version # 1.15.0
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    rustc --version # rustc 1.68.2 (9eb3afe9e 2023-03-27)
    rustup target add wasm32-unknown-unknown
    ```
