## SATNOGS downloader

### Usage

```
% export SATNOGS_API_TOKEN=...
% cargo build
% target/debug/satnogs_download -i 98849
```

Use `-h` to see available command line arguments. Start date will default to `2024-08-16` and end date will default
to tomorrow's date.

### More info

Satellite IDs:

* CUAVA-2: 98858
* WS-1: 98849

Satnogs downloader will download the data from observations
and store them in folders named by the satellite ID.

Folder will contain 2 files for each observation:
- *.raw: the raw data file
- *.url: file containing all URL links to individual data packets

All packets from a single observation will be concatenated into the raw data file.

Afterwards run `cuava-beacon-decoder -i "ID.raw" -o "ID.txt/json"` to generate JSON files.
