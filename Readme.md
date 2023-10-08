# airdrop-babylon-gateway

airdrop address filter with babylon gateway (radixdlt). You can download [here](https://file.io/eONhDrkusyUF).

## How to use

### One Command

cargo run input-csv-file -command- -command_filter- min max{optional} output-csv-file i file-for-invalid-addresses.
eg :

- cargo run add.csv resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd min 1000 tier1.csv i invalid.csv
- cargo run add.csv staking between 90 100 tier2.csv i invalid.csv

### Multiple Commands

you can also use multiple commands in one command and separated it with 'w'.
cargo run input-csv-file -command- -command_filter- min max{optional} output-csv-file w -command- -command_filter- min max{optional} output-csv-file.
eg :

- cargo run add.csv resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd min 100 output.csv w staking between 50 100 output1.csv i invalid.csv

#### command_filter

- 'min' get minimum amount of token.
- 'between' get range between min and max amount.

#### command_filter

- 'staking' get stake(lsu) info.
- '-resource_address-' get spesific token with resource_address.
